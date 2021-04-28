use tui::{backend::Backend, layout::Rect, widgets::Block, Frame};

use super::focus_block;
use crate::widget::*;

#[derive(Debug)]
pub struct Pane<'a> {
    widget: Widget<'a>,
    chunk_index: usize,
    title: String,
    id: String,
    chunk: Rect,
}

impl<'a> Pane<'a> {
    pub fn new(
        title: impl Into<String>,
        widget: Widget<'a>,
        chunk_index: usize,
        id: impl Into<String>,
    ) -> Self {
        Self {
            title: title.into(),
            widget,
            chunk_index,
            id: id.into(),
            chunk: Rect::default(),
        }
    }

    pub fn widget(&self) -> &Widget {
        &self.widget
    }

    pub fn widget_mut(&mut self) -> &mut Widget<'a> {
        &mut self.widget
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn chunk_index(&self) -> usize {
        self.chunk_index
    }

    pub fn next_item(&mut self, index: usize) {
        self.widget.select_next(index)
    }

    pub fn prev_item(&mut self, index: usize) {
        self.widget.select_prev(index)
    }

    pub fn set_items(&mut self, items: WidgetItem) {
        self.widget.set_items(items);
    }

    pub fn is_selected(&self, rhs: &Pane) -> bool {
        std::ptr::eq(self, rhs)
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn update_chunk(&mut self, chunk: Rect) {
        self.chunk = chunk;

        self.widget.update_area(self.block(false).inner(chunk));
    }

    pub fn chunk(&self) -> Rect {
        self.chunk
    }

    pub fn block(&self, selected: bool) -> Block {
        focus_block(&self.title, selected)
    }
}

impl Pane<'_> {
    pub fn render<B>(&mut self, f: &mut Frame<B>, selected: bool)
    where
        B: Backend,
    {
        self.widget
            .render(f, focus_block(&self.title, selected), self.chunk);
    }
}
