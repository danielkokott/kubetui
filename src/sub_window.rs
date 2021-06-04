use tui_wrapper::crossterm::event::MouseEvent;
use tui_wrapper::tui::{
    backend::Backend,
    layout::Rect,
    text::Span,
    widgets::{Block, Clear},
    Frame,
};

use component::*;
use tui_wrapper::widget::{WidgetItem, WidgetTrait};
use tui_wrapper::{child_window_chunk, focus_title_style, Pane};

pub trait PaneTrait {
    type Item;
    fn id(&self) -> &str;
    fn update_chunks(&mut self, chunk: Rect);
    fn select_next_pane(&mut self) {}
    fn select_prev_pane(&mut self) {}
    fn select_next_item(&mut self) {}
    fn select_prev_item(&mut self) {}
    fn select_first_item(&mut self) {}
    fn select_last_item(&mut self) {}
    fn set_items(&mut self, id: &str, items: WidgetItem);
    fn get_item(&self, _: &str) -> Option<WidgetItem> {
        None
    }
    fn render<B: Backend>(&mut self, f: &mut Frame<B>);
    fn get(&self) -> &Self::Item;
    fn get_mut(&mut self) -> &mut Self::Item;
    fn on_mouse_event(&mut self, ev: MouseEvent);
}

pub struct SubWindow<'a, P> {
    id: String,
    title: String,
    chunk: Rect,
    pane: P,
    block: Option<Block<'a>>,
}

impl<'a, P> SubWindow<'a, P>
where
    P: PaneTrait,
{
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        pane: P,
        block: Option<Block<'a>>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            chunk: Rect::default(),
            pane,
            block,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn update_chunks(&mut self, chunk: Rect) {
        self.chunk = child_window_chunk(80, 80, chunk);

        let inner_chunk = match self.block {
            Some(ref b) => b.inner(b.inner(self.chunk)),
            None => self.chunk,
        };

        self.pane.update_chunks(inner_chunk);
    }

    pub fn render<B: Backend>(&mut self, f: &mut Frame<B>) {
        f.render_widget(Clear, self.chunk);

        if let Some(ref b) = self.block {
            f.render_widget(
                b.clone()
                    .title(Span::styled(self.title.as_str(), focus_title_style(true)))
                    .title_offset(1),
                b.inner(self.chunk),
            );
        }

        self.pane.render(f);
    }

    pub fn pane(&self) -> &P::Item {
        self.pane.get()
    }

    pub fn pane_mut(&mut self) -> &mut P::Item {
        self.pane.get_mut()
    }

    pub fn on_mouse_event(&mut self, ev: MouseEvent) {
        self.pane.on_mouse_event(ev);
    }
}

// impl
impl<'a> PaneTrait for Pane<'a> {
    type Item = Pane<'a>;

    fn id(&self) -> &str {
        self.id()
    }

    fn update_chunks(&mut self, chunk: Rect) {
        self.update_chunk(chunk);
    }

    fn select_next_pane(&mut self) {}

    fn select_prev_pane(&mut self) {}

    fn select_first_item(&mut self) {
        self.select_first_item();
    }

    fn select_last_item(&mut self) {
        self.select_last_item();
    }

    fn set_items(&mut self, _id: &str, items: WidgetItem) {
        self.widget_mut().set_items(items);
    }

    fn render<B: Backend>(&mut self, f: &mut Frame<B>) {
        self.render(f, true);
    }

    fn get_item(&self, _id: &str) -> Option<WidgetItem> {
        self.widget().get_item()
    }

    fn select_next_item(&mut self) {
        self.select_next_item(1)
    }

    fn select_prev_item(&mut self) {
        self.select_prev_item(1)
    }

    fn get(&self) -> &Self::Item {
        self
    }

    fn get_mut(&mut self) -> &mut Self::Item {
        self
    }

    fn on_mouse_event(&mut self, ev: MouseEvent) {
        self.on_mouse_event(ev);
    }
}

impl<'a> PaneTrait for MultipleSelect<'a> {
    type Item = MultipleSelect<'a>;

    fn update_chunks(&mut self, chunk: Rect) {
        self.update_chunk(chunk)
    }

    fn select_next_pane(&mut self) {
        self.toggle_focus()
    }

    fn select_prev_pane(&mut self) {
        self.toggle_focus()
    }

    fn select_next_item(&mut self) {
        self.select_next_item()
    }

    fn select_prev_item(&mut self) {
        self.select_prev_item()
    }

    fn set_items(&mut self, _id: &str, items: WidgetItem) {
        self.set_list_items(items.array())
    }

    fn render<B: Backend>(&mut self, f: &mut Frame<B>) {
        self.render(f);
    }

    fn get(&self) -> &Self::Item {
        self
    }

    fn get_mut(&mut self) -> &mut Self::Item {
        self
    }

    fn id(&self) -> &str {
        self.id()
    }

    fn on_mouse_event(&mut self, ev: MouseEvent) {
        self.on_mouse_event(ev);
    }
}

impl<'a> PaneTrait for SingleSelect<'a> {
    type Item = SingleSelect<'a>;

    fn select_next_item(&mut self) {
        self.select_next_item()
    }

    fn select_prev_item(&mut self) {
        self.select_prev_item()
    }

    fn set_items(&mut self, _id: &str, items: WidgetItem) {
        self.set_items(items.array())
    }

    fn get_item(&self, _id: &str) -> Option<WidgetItem> {
        self.get_item()
    }

    fn render<B: Backend>(&mut self, f: &mut Frame<B>) {
        self.render(f)
    }

    fn get(&self) -> &Self::Item {
        self
    }

    fn get_mut(&mut self) -> &mut Self::Item {
        self
    }

    fn id(&self) -> &str {
        self.id()
    }

    fn update_chunks(&mut self, chunk: Rect) {
        self.update_chunk(chunk);
    }
    fn on_mouse_event(&mut self, ev: MouseEvent) {
        self.on_mouse_event(ev);
    }
}
