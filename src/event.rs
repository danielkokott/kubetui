use super::{util::*, window::*};

#[allow(unused_imports)]
use chrono::{DateTime, Duration, Utc};

#[allow(unused_imports)]
use std::sync::{
    mpsc::{self, Receiver, Sender},
    Arc, RwLock,
};
use std::thread;
use std::time;

#[allow(unused_imports)]
use tokio::runtime::Runtime;

#[allow(unused_imports)]
use std::{
    error::Error,
    io::{self, stdout, Write},
};

#[allow(unused_imports)]
use crossterm::{
    event::{
        self, poll, read, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode,
        KeyEvent, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

#[allow(unused_imports)]
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Corner, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets, Frame, Terminal,
};

#[allow(unused_imports)]
use k8s_openapi::{
    api::core::v1::{Namespace, Pod, PodStatus},
    apimachinery::pkg::apis::meta::v1::Time,
};
use kube::{
    api::{ListParams, LogParams, Meta},
    config::Kubeconfig,
    Api, Client,
};

pub enum Event {
    Input(KeyEvent),
    Kube(Kube),
    Tick,
    Resize,
    Mouse,
}

pub enum Kube {
    Pod(Vec<String>),
    Namespace(Option<Vec<String>>),
    LogRequest(String),
    LogResponse(Option<Vec<String>>),
}

pub struct PodInfo {
    name: String,
    phase: String,
    age: String,
}

impl PodInfo {
    fn new(name: String, phase: String, age: String) -> Self {
        Self { name, phase, age }
    }

    fn to_string(&self, width: usize) -> String {
        format!(
            "{:width$} {}    {}",
            self.name,
            self.phase,
            self.age,
            width = width + 4
        )
    }
}

fn draw_tab<B: Backend, W: Widget>(
    f: &mut Frame<B>,
    chunk: Rect,
    tabs: &Vec<Tab<W>>,
    index: usize,
) {
    let titles: Vec<Spans> = tabs
        .iter()
        .map(|t| Spans::from(format!(" {} ", t.title())))
        .collect();

    let block = widgets::Block::default().style(Style::default());

    let tabs = widgets::Tabs::new(titles)
        .block(block)
        .select(index)
        .highlight_style(Style::default().fg(Color::White).bg(Color::LightBlue));

    f.render_widget(tabs, chunk);
}

fn generate_title(title: &str, border_color: Color, selected: bool) -> Spans {
    let prefix = if selected { "✔︎ " } else { "──" };
    Spans::from(vec![
        Span::styled("─", Style::default()),
        Span::styled(prefix, Style::default().fg(border_color)),
        Span::styled(
            title,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
    ])
}

fn draw_panes<B: Backend, W: Widget>(f: &mut Frame<B>, area: Rect, tab: &Tab<W>) {
    let chunks = tab.chunks(area);

    for pane in tab.panes() {
        let selected = pane.selected(tab.selected_pane());

        let border_color = if selected {
            Color::White
        } else {
            Color::DarkGray
        };

        let block = widgets::Block::default()
            .title(generate_title(pane.title(), border_color, selected))
            .borders(widgets::Borders::ALL)
            .border_style(Style::default().fg(border_color));

        match pane.ty() {
            Type::POD => {
                draw_list(
                    f,
                    block,
                    chunks[pane.chunk_index()],
                    &pane.widget().items(),
                    &mut pane.widget().list_state().borrow_mut(),
                    selected,
                );
            }
            Type::LOG => {
                draw_paragraph(f, block, chunks[pane.chunk_index()], &pane.widget().items());
            }
            Type::NONE => {}
        }
    }
}

fn draw_list<B: Backend>(
    f: &mut Frame<B>,
    block: widgets::Block,
    area: Rect,
    items: &Vec<String>,
    state: &mut widgets::ListState,
    selected: bool,
) {
    let items: Vec<widgets::ListItem> = items
        .iter()
        .map(|i| widgets::ListItem::new(i.as_ref()))
        .collect();

    let style = if selected {
        Style::default().add_modifier(Modifier::REVERSED)
    } else {
        Style::default()
    };

    let li = widgets::List::new(items)
        .block(block)
        .style(Style::default())
        .highlight_style(style);

    f.render_stateful_widget(li, area, state);
}

fn draw_paragraph<B: Backend>(
    f: &mut Frame<B>,
    block: widgets::Block,
    area: Rect,
    items: &Vec<String>,
) {
    let text: Vec<Spans> = items
        .iter()
        .map(|item| Spans::from(Span::raw(item)))
        .collect();

    let paragraph = widgets::Paragraph::new(text)
        .block(block)
        .style(Style::default().fg(Color::White).bg(Color::Black))
        .scroll((0, 0))
        .wrap(widgets::Wrap { trim: true });

    f.render_widget(paragraph, area);
}

fn draw_datetime<B: Backend>(f: &mut Frame<B>, area: Rect) {
    let block = widgets::Block::default().style(Style::default());

    let text = Spans::from(vec![Span::raw(format!(
        " {}",
        Utc::now().format("%Y年%m月%d日 %H時%M分%S秒")
    ))]);

    let paragraph = widgets::Paragraph::new(text).block(block);

    f.render_widget(paragraph, area);
}

pub fn draw<B: Backend, W: Widget>(f: &mut Frame<B>, window: &mut Window<W>) {
    let areas = window.chunks(f.size());

    draw_tab(f, areas[0], &window.tabs(), window.selected_tab_index());

    draw_panes(f, areas[1], window.selected_tab());

    draw_datetime(f, areas[2]);
}

async fn get_pod_info(client: Client, namespace: &str) -> Vec<String> {
    let pods: Api<Pod> = Api::namespaced(client, namespace);
    let lp = ListParams::default();

    let pods_list = pods.list(&lp).await.unwrap();

    let max_name_len = pods_list
        .iter()
        .max_by(|r, l| r.name().len().cmp(&l.name().len()))
        .unwrap()
        .name()
        .len();

    let current_datetime: DateTime<Utc> = Utc::now();

    let mut ret: Vec<String> = Vec::new();
    for p in pods_list {
        let meta = Meta::meta(&p);
        let status = &p.status;
        let name = meta.name.clone().unwrap();

        let phase = match status {
            Some(s) => s.phase.clone().unwrap(),
            None => "Unknown".to_string(),
        };
        let creation_timestamp: DateTime<Utc> = match &meta.creation_timestamp {
            Some(ref time) => time.0,
            None => current_datetime,
        };
        let duration: Duration = current_datetime - creation_timestamp;

        ret.push(PodInfo::new(name, phase, age(&duration)).to_string(max_name_len));
    }
    ret
}

pub fn read_key(tx: Sender<Event>) {
    loop {
        match read().unwrap() {
            CEvent::Key(ev) => tx.send(Event::Input(ev)).unwrap(),
            CEvent::Mouse(_) => tx.send(Event::Mouse).unwrap(),
            CEvent::Resize(_, _) => tx.send(Event::Resize).unwrap(),
        }
    }
}

// TODO: spawnを削除する <20-02-21, yourname> //
fn get_namespace_list() -> Option<Vec<String>> {
    let th = thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        rt.block_on(async move {
            let client = Client::try_default().await.unwrap();
            let namespaces: Api<Namespace> = Api::all(client);

            let lp = ListParams::default();

            let ns_list = namespaces.list(&lp).await.unwrap();

            ns_list.iter().map(|ns| ns.name()).collect::<Vec<String>>()
        })
    });

    Some(th.join().unwrap())
}

async fn get_logs(client: Client, namespace: &str, pod_name: &str) -> Option<Vec<String>> {
    let pod: Api<Pod> = Api::namespaced(client, namespace);
    let lp = LogParams::default();
    let logs = pod.logs(pod_name, &lp).await;

    match logs {
        Ok(logs) => Some(logs.lines().map(String::from).collect()),
        Err(_) => None,
    }
}

pub fn kube_process(tx: Sender<Event>, rx: Receiver<Event>) {
    let rt = Runtime::new().unwrap();

    rt.block_on(async move {
        let kubeconfig = Kubeconfig::read().unwrap();
        let current_context = kubeconfig.current_context.unwrap();

        let current_context = kubeconfig
            .contexts
            .iter()
            .find(|n| n.name == current_context);

        let namespace = Arc::new(RwLock::new(
            current_context.unwrap().clone().context.namespace.unwrap(),
        ));

        let tx_pod = tx.clone();
        let tx_log = tx.clone();
        let tx_ns = tx.clone();

        let namespace_event_loop = Arc::clone(&namespace);
        let namespace_pod_loop = Arc::clone(&namespace);

        let event_loop = tokio::spawn(async move {
            let client = Client::try_default().await.unwrap();
            loop {
                let ev = rx.recv().unwrap();
                match ev {
                    Event::Kube(ev) => match ev {
                        Kube::Namespace(_) => tx_ns
                            .send(Event::Kube(Kube::Namespace(get_namespace_list())))
                            .unwrap(),

                        Kube::LogRequest(pod_name) => {
                            let client_clone = client.clone();
                            let namespace = namespace_event_loop.read().unwrap().clone();
                            let logs = get_logs(client_clone, &namespace, &pod_name).await;

                            tx_log.send(Event::Kube(Kube::LogResponse(logs))).unwrap();
                        }
                        _ => {
                            unreachable!()
                        }
                    },
                    _ => {
                        unreachable!()
                    }
                }
            }
        });

        let pod_loop = tokio::spawn(async move {
            let mut interval = tokio::time::interval(time::Duration::from_secs(1));
            let client = Client::try_default().await.unwrap();
            loop {
                interval.tick().await;
                let namespace = namespace_pod_loop.read().unwrap().clone();
                let pod_info = get_pod_info(client.clone(), &namespace).await;
                tx_pod.send(Event::Kube(Kube::Pod(pod_info))).unwrap();
            }
        });

        event_loop.await.unwrap();
        pod_loop.await.unwrap();
    });
}

pub fn tick(tx: Sender<Event>, rate: time::Duration) {
    let rt = Runtime::new().unwrap();

    rt.block_on(async move {
        let mut interval = tokio::time::interval(rate);
        loop {
            interval.tick().await;

            tx.send(Event::Tick).unwrap();
        }
    });
}
