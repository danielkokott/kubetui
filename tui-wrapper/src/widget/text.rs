use tui::layout::Rect;
use tui::style::Style;
use tui::text::{Span, Spans};
use tui::widgets::{Block, Paragraph};

use super::ansi::*;
use super::WidgetTrait;

const BORDER_WIDTH: usize = 2;
const ESC_LEN: usize = 2; // "\x1b["

pub struct Text<'a> {
    items: Vec<String>,
    state: TextState,
    spans: Vec<Spans<'a>>,
    row_size: u64,
}

#[derive(Clone, Copy)]
pub struct TextState {
    scroll: u64,
}

impl TextState {
    fn select(&mut self, index: u64) {
        self.scroll = index;
    }
    fn selected(&self) -> u64 {
        self.scroll
    }
}

impl Default for TextState {
    fn default() -> Self {
        Self { scroll: 0 }
    }
}

// ステート
impl Text<'_> {
    pub fn new(items: Vec<String>) -> Self {
        Self {
            items,
            state: TextState::default(),
            spans: vec![Spans::default()],
            row_size: 0,
        }
    }

    pub fn select(&mut self, scroll: u64) {
        self.state.select(scroll);
    }

    pub fn state(&self) -> TextState {
        self.state
    }

    pub fn selected(&self) -> u64 {
        self.state.selected()
    }
    pub fn scroll_top(&mut self) {
        self.state.select(0);
    }

    pub fn scroll_bottom(&mut self) {
        self.state.select(self.row_size);
    }

    pub fn is_bottom(&self) -> bool {
        self.selected() == self.row_size
    }

    pub fn scroll_down(&mut self, index: u64) {
        (0..index).for_each(|_| self.select_next(1));
    }

    pub fn scroll_up(&mut self, index: u64) {
        (0..index).for_each(|_| self.select_prev(1));
    }
}

impl Default for Text<'_> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            state: TextState::default(),
            spans: Vec::new(),
            row_size: 0,
        }
    }
}

// コンテンツ操作
impl<'a> Text<'a> {
    pub fn items(&self) -> &Vec<String> {
        &self.items
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn spans(&self) -> &Vec<Spans> {
        &self.spans
    }

    pub fn widget(&self, block: Block<'a>, area: Rect) -> Paragraph<'a> {
        let area = block.inner(area);

        let start = self.state.selected() as usize;

        let end = if self.spans.len() < area.height as usize {
            self.spans.len()
        } else {
            start + area.height as usize
        };

        Paragraph::new(self.spans[start..end].to_vec())
            .block(block)
            .style(Style::default())
    }

    pub fn row_size(&self) -> u64 {
        self.row_size
    }

    pub fn add_item(&mut self, item: impl Into<String>) {
        self.items.push(item.into());
    }

    pub fn append_items(&mut self, items: &Vec<String>, width: u64, height: u64) {
        self.items.append(&mut items.clone());

        let w = width as usize - BORDER_WIDTH;
        let wrapped = wrap(items, w);

        self.spans.append(&mut generate_spans(&wrapped));

        self.update_rows_size(height);
    }

    pub fn update_spans(&mut self, width: u64) {
        let w = width as usize - BORDER_WIDTH;
        let lines = wrap(&self.items, w);

        self.spans = generate_spans(&lines);
    }

    pub fn update_rows_size(&mut self, height: u64) {
        let mut count = self.spans.len() as u64;

        let height = height - BORDER_WIDTH as u64; // 2: border-line

        if height < count {
            count -= height;
        } else {
            count = 0
        }

        self.row_size = count;
    }
}

impl WidgetTrait for Text<'_> {
    fn selectable(&self) -> bool {
        true
    }

    fn select_next(&mut self, index: usize) {
        let mut i = self.state.selected();

        if self.row_size <= i {
            i = self.row_size;
        } else {
            i = i + index as u64;
        }

        self.state.select(i);
    }

    fn select_prev(&mut self, index: usize) {
        let mut i = self.state.selected();
        if i == 0 {
            i = 0;
        } else {
            i = i - index as u64;
        }
        self.state.select(i);
    }

    fn select_first(&mut self) {
        self.state.select(0);
    }
    fn select_last(&mut self) {
        self.state.select(self.row_size);
    }

    fn set_items(&mut self, items: Vec<String>) {
        self.state.select(0);
        self.items = items;
    }
}

pub fn wrap(lines: &Vec<String>, width: usize) -> Vec<String> {
    let mut ret = Vec::new();

    for line in lines.iter() {
        ret.append(&mut wrap_line(line, width));
    }

    ret
}

fn wrap_line(text: &String, width: usize) -> Vec<String> {
    let mut ret = Vec::new();
    if text.len() == 0 {
        ret.push("".to_string());
        return ret;
    }

    let lines = text.lines();

    for line in lines {
        let len = line.chars().count();
        if width < len {
            let mut word_count = 0;
            let mut start = 0;
            let mut into_escape = false;
            // 線形探索
            let chars = line.chars();
            for (i, c) in chars.enumerate() {
                // skip \x1b[...m
                if c == '\x1b' {
                    into_escape = true;
                    continue;
                }

                if into_escape && c == 'm' {
                    into_escape = false;
                    continue;
                }

                if into_escape {
                    continue;
                }

                word_count += 1;
                if word_count == width {
                    ret.push(String::from(&line[start..=i]));

                    start = i + 1;

                    word_count = 0;
                }
            }

            if 0 < word_count {
                ret.push(String::from(&line[start..]));
            }
        } else {
            ret.push(line.to_string());
        }
    }
    ret
}

pub fn generate_spans<'a>(lines: &Vec<String>) -> Vec<Spans<'a>> {
    lines
        .iter()
        .map(|line| {
            let mut span_vec: Vec<Span> = vec![];

            let mut l = &line[..];

            if let Some(escape_start) = l.find("\x1b[") {
                if 0 < escape_start {
                    span_vec.push(Span::raw(String::from(&l[..escape_start])));
                    l = &l[escape_start..];
                }
            }

            let mut found = false;
            while let Some(escape_start) = l.find("\x1b[") {
                found = true;

                let escape_end = l[escape_start..].find("m").unwrap();

                // \x1b[<xxx>m xxxで示したセミコロン区切りの数字を抜き出す
                let escape = &l[(escape_start + ESC_LEN)..escape_end];

                // skip m  \x1b[<xx>m
                l = &l[(escape_end + 1)..];

                // 次のescape sequenceを探す
                // なければ末尾まで
                let content_end;
                if let Some(next_esc_index) = l.find("\x1b[") {
                    content_end = next_esc_index;
                } else {
                    content_end = l.len();
                }

                span_vec.push(Span::styled(
                    String::from(&l[..content_end]),
                    generate_style_from_ansi_color(&escape),
                ));

                l = &l[content_end..];
            }

            if found == false {
                Spans::from(l.to_string())
            } else {
                Spans::from(span_vec)
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use tui::style::{Color, Modifier, Style};

    #[test]
    fn wrap_only_newline() {
        let text = vec!["hoge".to_string(), "".to_string(), "hoge".to_string()];

        assert_eq!(
            wrap(&text, 100),
            vec!["hoge".to_string(), "".to_string(), "hoge".to_string()]
        );
    }

    #[test]
    fn wrap_unwrap() {
        let text = vec!["aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string()];

        assert_eq!(
            wrap(&text, 100),
            vec!["aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string()]
        );
    }

    #[test]
    fn wrap_short() {
        let text = vec!["aaaaaaaaaaaaaaa".to_string()];

        assert_eq!(
            wrap(&text, 10),
            vec!["aaaaaaaaaa".to_string(), "aaaaa".to_string()]
        );
    }
    #[test]
    fn wrap_too_long_0() {
        let text = vec!["aaaaaaaaaaaaaa".to_string()];

        assert_eq!(
            wrap(&text, 5),
            vec!["aaaaa".to_string(), "aaaaa".to_string(), "aaaa".to_string(),]
        );
    }

    #[test]
    fn wrap_too_long_1() {
        let text = vec!["123456789\n123456789\n123456789\n123456789\n123456789\n123456789\n123456789\n123456789\n".to_string()];

        assert_eq!(
            wrap(&text, 12),
            vec![
                "123456789".to_string(),
                "123456789".to_string(),
                "123456789".to_string(),
                "123456789".to_string(),
                "123456789".to_string(),
                "123456789".to_string(),
                "123456789".to_string(),
                "123456789".to_string(),
            ]
        );
    }

    #[test]
    fn spans() {
        let text = vec![
            "> taskbox@0.1.0 start /app",
            "> react-scripts start",
            "",
            "\x1b[34mℹ\x1b[39m \x1b[90m｢wds｣\x1b[39m: Project is running at http://10.1.157.9/",
            "\x1b[34mℹ\x1b[39m \x1b[90m｢wds｣\x1b[39m: webpack output is served from",
            "\x1b[34mℹ\x1b[39m \x1b[90m｢wds｣\x1b[39m: Content not from webpack is served from /app/public",
            "\x1b[34mℹ\x1b[39m \x1b[90m｢wds｣\x1b[39m: 404s will fallback to /",
            "Starting the development server...",
            "",
            "Compiled successfully!",
            "",
            "You can now view taskbox in the browser.",
            "",
            "  Local:            http://localhost:3000",
            "  On Your Network:  http://10.1.157.9:3000",
            "",
            "Note that the development build is not optimized.",
            "To create a production build, use npm run build.",
        ];

        let wrapped = wrap(&text.iter().cloned().map(String::from).collect(), 100);

        let expected = vec![
            Spans::from("> taskbox@0.1.0 start /app"),
            Spans::from("> react-scripts start"),
            Spans::from(""),
            Spans::from(vec![
                Span::styled("ℹ", Style::default().fg(Color::Blue)),
                Span::styled(" ", Style::default().fg(Color::Reset)),
                Span::styled("｢wds｣", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    ": Project is running at http://10.1.157.9/",
                    Style::default().fg(Color::Reset),
                ),
            ]),
            Spans::from(vec![
                Span::styled("ℹ", Style::default().fg(Color::Blue)),
                Span::styled(" ", Style::default().fg(Color::Reset)),
                Span::styled("｢wds｣", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    ": webpack output is served from",
                    Style::default().fg(Color::Reset),
                ),
            ]),
            Spans::from(vec![
                Span::styled("ℹ", Style::default().fg(Color::Blue)),
                Span::styled(" ", Style::default().fg(Color::Reset)),
                Span::styled("｢wds｣", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    ": Content not from webpack is served from /app/public",
                    Style::default().fg(Color::Reset),
                ),
            ]),
            Spans::from(vec![
                Span::styled("ℹ", Style::default().fg(Color::Blue)),
                Span::styled(" ", Style::default().fg(Color::Reset)),
                Span::styled("｢wds｣", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    ": 404s will fallback to /",
                    Style::default().fg(Color::Reset),
                ),
            ]),
            Spans::from("Starting the development server..."),
            Spans::from(""),
            Spans::from("Compiled successfully!"),
            Spans::from(""),
            Spans::from("You can now view taskbox in the browser."),
            Spans::from(""),
            Spans::from("  Local:            http://localhost:3000"),
            Spans::from("  On Your Network:  http://10.1.157.9:3000"),
            Spans::from(""),
            Spans::from("Note that the development build is not optimized."),
            Spans::from("To create a production build, use npm run build."),
        ];

        let result = generate_spans(&wrapped);
        for (i, l) in result.iter().enumerate() {
            assert_eq!(*l, expected[i]);
        }
    }

    #[test]
    fn generate_spans_color_3_4bit_fg() {
        let text = vec!["hoge\x1b[33mhoge\x1b[39m".to_string()];

        assert_eq!(
            generate_spans(&text),
            vec![Spans::from(vec![
                Span::raw("hoge"),
                Span::styled("hoge", Style::default().fg(Color::Yellow)),
                Span::styled("", Style::default().fg(Color::Reset)),
            ])]
        )
    }

    #[test]
    fn generate_spans_color_3_4bit_fg_bold() {
        let text = vec!["\x1b[1;33mhoge\x1b[39m".to_string()];

        assert_eq!(
            generate_spans(&text),
            vec![Spans::from(vec![
                Span::styled(
                    "hoge",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                ),
                Span::styled("", Style::default().fg(Color::Reset)),
            ])]
        )
    }

    #[test]
    fn generate_spans_color_8bit_fg() {
        let text = vec!["\x1b[38;5;33mhoge\x1b[39m".to_string()];

        assert_eq!(
            generate_spans(&text),
            vec![Spans::from(vec![
                Span::styled("hoge", Style::default().fg(Color::Indexed(33))),
                Span::styled("", Style::default().fg(Color::Reset)),
            ])]
        )
    }

    #[test]
    fn generate_spans_color_8bit_fg_bold() {
        let text = vec!["\x1b[1;38;5;33mhoge\x1b[39m".to_string()];

        assert_eq!(
            generate_spans(&text),
            vec![Spans::from(vec![
                Span::styled(
                    "hoge",
                    Style::default()
                        .fg(Color::Indexed(33))
                        .add_modifier(Modifier::BOLD)
                ),
                Span::styled("", Style::default().fg(Color::Reset)),
            ])]
        )
    }

    #[test]
    fn generate_spans_color_24bit_fg() {
        let text = vec!["\x1b[38;2;33;10;10mhoge\x1b[39m".to_string()];

        assert_eq!(
            generate_spans(&text),
            vec![Spans::from(vec![
                Span::styled("hoge", Style::default().fg(Color::Rgb(33, 10, 10))),
                Span::styled("", Style::default().fg(Color::Reset)),
            ])]
        )
    }

    #[test]
    fn generate_spans_color_24bit_fg_bold() {
        let text = vec!["\x1b[1;38;2;33;10;10mhoge\x1b[39m".to_string()];

        assert_eq!(
            generate_spans(&text),
            vec![Spans::from(vec![
                Span::styled(
                    "hoge",
                    Style::default()
                        .fg(Color::Rgb(33, 10, 10))
                        .add_modifier(Modifier::BOLD)
                ),
                Span::styled("", Style::default().fg(Color::Reset)),
            ])]
        )
    }

    #[test]
    fn generate_spans_color_3_4bit_bg() {
        let text = vec!["\x1b[43mhoge\x1b[49m".to_string()];

        assert_eq!(
            generate_spans(&text),
            vec![Spans::from(vec![
                Span::styled("hoge", Style::default().bg(Color::Yellow)),
                Span::styled("", Style::default().bg(Color::Reset)),
            ])]
        )
    }

    #[test]
    fn generate_spans_color_3_4bit_bg_bold() {
        let text = vec!["\x1b[1;43mhoge\x1b[49m".to_string()];

        assert_eq!(
            generate_spans(&text),
            vec![Spans::from(vec![
                Span::styled(
                    "hoge",
                    Style::default()
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                ),
                Span::styled("", Style::default().bg(Color::Reset)),
            ])]
        );

        let text = vec!["\x1b[43;1mhoge\x1b[49m".to_string()];

        assert_eq!(
            generate_spans(&text),
            vec![Spans::from(vec![
                Span::styled(
                    "hoge",
                    Style::default()
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                ),
                Span::styled("", Style::default().bg(Color::Reset)),
            ])]
        );
    }

    #[test]
    fn generate_spans_color_8bit_bg() {
        let text = vec!["\x1b[48;5;33mhoge\x1b[49m".to_string()];

        assert_eq!(
            generate_spans(&text),
            vec![Spans::from(vec![
                Span::styled("hoge", Style::default().bg(Color::Indexed(33))),
                Span::styled("", Style::default().bg(Color::Reset)),
            ])]
        );
    }

    #[test]
    fn generate_spans_color_8bit_bg_bold() {
        let text = vec!["\x1b[1;48;5;33mhoge\x1b[49m".to_string()];

        assert_eq!(
            generate_spans(&text),
            vec![Spans::from(vec![
                Span::styled(
                    "hoge",
                    Style::default()
                        .bg(Color::Indexed(33))
                        .add_modifier(Modifier::BOLD)
                ),
                Span::styled("", Style::default().bg(Color::Reset)),
            ])]
        );

        let text = vec!["\x1b[48;5;33;1mhoge\x1b[49m".to_string()];

        assert_eq!(
            generate_spans(&text),
            vec![Spans::from(vec![
                Span::styled(
                    "hoge",
                    Style::default()
                        .bg(Color::Indexed(33))
                        .add_modifier(Modifier::BOLD)
                ),
                Span::styled("", Style::default().bg(Color::Reset)),
            ])]
        );
    }

    #[test]
    fn generate_spans_color_24bit_bg() {
        let text = vec!["\x1b[48;2;33;10;10mhoge\x1b[49m".to_string()];

        assert_eq!(
            generate_spans(&text),
            vec![Spans::from(vec![
                Span::styled("hoge", Style::default().bg(Color::Rgb(33, 10, 10))),
                Span::styled("", Style::default().bg(Color::Reset)),
            ])]
        );
    }

    #[test]
    fn generate_spans_color_24bit_bg_bold() {
        let text = vec!["\x1b[1;48;2;33;10;10mhoge\x1b[49m".to_string()];

        assert_eq!(
            generate_spans(&text),
            vec![Spans::from(vec![
                Span::styled(
                    "hoge",
                    Style::default()
                        .bg(Color::Rgb(33, 10, 10))
                        .add_modifier(Modifier::BOLD)
                ),
                Span::styled("", Style::default().bg(Color::Reset)),
            ])]
        );

        let text = vec!["\x1b[48;2;33;10;10;1mhoge\x1b[49m".to_string()];

        assert_eq!(
            generate_spans(&text),
            vec![Spans::from(vec![
                Span::styled(
                    "hoge",
                    Style::default()
                        .bg(Color::Rgb(33, 10, 10))
                        .add_modifier(Modifier::BOLD)
                ),
                Span::styled("", Style::default().bg(Color::Reset)),
            ])]
        );
    }

    #[test]
    fn generate_spans_color_24bit_rainbow() {
        let rainbow = vec!["[48;2;0;0;0m [48;2;1;0;0m [48;2;2;0;0m [48;2;3;0;0m [48;2;4;0;0m [48;2;5;0;0m [48;2;6;0;0m [48;2;7;0;0m [48;2;8;0;0m [48;2;9;0;0m [48;2;10;0;0m [0m".to_string()];

        let wrapped = wrap(&rainbow, 3);

        println!("{:?}", wrapped);

        assert_eq!(
            generate_spans(&wrapped),
            vec![
                Spans::from(vec![
                    Span::styled(" ", Style::default().bg(Color::Rgb(0, 0, 0))),
                    Span::styled(" ", Style::default().bg(Color::Rgb(1, 0, 0))),
                    Span::styled(" ", Style::default().bg(Color::Rgb(2, 0, 0)))
                ]),
                Spans::from(vec![
                    Span::styled(" ", Style::default().bg(Color::Rgb(3, 0, 0))),
                    Span::styled(" ", Style::default().bg(Color::Rgb(4, 0, 0))),
                    Span::styled(" ", Style::default().bg(Color::Rgb(5, 0, 0)))
                ]),
                Spans::from(vec![
                    Span::styled(" ", Style::default().bg(Color::Rgb(6, 0, 0))),
                    Span::styled(" ", Style::default().bg(Color::Rgb(7, 0, 0))),
                    Span::styled(" ", Style::default().bg(Color::Rgb(8, 0, 0)))
                ]),
                Spans::from(vec![
                    Span::styled(" ", Style::default().bg(Color::Rgb(9, 0, 0))),
                    Span::styled(" ", Style::default().bg(Color::Rgb(10, 0, 0))),
                    Span::styled("", Style::reset())
                ]),
            ]
        );
    }
}
