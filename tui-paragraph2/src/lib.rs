// version 0.14を改変
// https://github.com/fdehau/tui-rs/blob/master/src/widgets/paragraph.rs

use tui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{StyledGrapheme, Text},
    widgets::{Block, Widget},
};

mod reflow;

use reflow::{LineComposer, LineTruncator};

use std::iter;
use unicode_width::UnicodeWidthStr;

#[derive(Debug, Clone)]
pub struct Paragraph2<'a> {
    block: Option<Block<'a>>,
    style: Style,
    text: Text<'a>,
}

impl<'a> Paragraph2<'a> {
    pub fn new(text: impl Into<Text<'a>>) -> Self {
        Self {
            block: None,
            style: Default::default(),
            text: text.into(),
        }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl<'a> Widget for Paragraph2<'a> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        buf.set_style(area, self.style);

        let text_area = match self.block.take() {
            Some(b) => {
                let inner_area = b.inner(area);
                b.render(area, buf);
                inner_area
            }
            None => area,
        };

        if text_area.height < 1 {
            return;
        }

        let style = self.style;

        let mut styled = self.text.lines.iter().flat_map(|spans| {
            spans
                .0
                .iter()
                .flat_map(|span| span.styled_graphemes(style))
                .chain(iter::once(StyledGrapheme {
                    symbol: "\n",
                    style: self.style,
                }))
        });

        let mut line_composer = Box::new(LineTruncator::new(&mut styled, text_area.width));

        let mut y = 0;

        while let Some((current_line, _)) = line_composer.next_line() {
            let mut x = 0;

            for StyledGrapheme { symbol, style } in current_line {
                buf.get_mut(text_area.left() + x, text_area.top() + y)
                    .set_symbol(if symbol.is_empty() { " " } else { symbol })
                    .set_style(*style);
                x += symbol.width() as u16;
            }
            y += 1;
            if text_area.height <= y {
                break;
            }
        }
    }
}
