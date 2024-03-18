use crossbeam::channel::Sender;
use ratatui::prelude::Constraint;

use std::{cell::RefCell, rc::Rc};

use crate::{
    action::view_id,
    clipboard::Clipboard,
    message::Message,
    ui::{
        tab::{LayoutElement, NestedLayoutElement, NestedWidgetLayout},
        widget::Widget,
        Tab,
    },
};

use super::{popup::popup_widget, widget::list_widget};

pub struct ListTab {
    pub tab: Tab<'static>,
    pub popup: Widget<'static>,
}

impl ListTab {
    pub fn new(
        title: &'static str,
        tx: &Sender<Message>,
        clipboard: &Option<Rc<RefCell<Clipboard>>>,
    ) -> Self {
        let list_widget = list_widget(tx, clipboard);

        ListTab {
            tab: Tab::new(
                view_id::tab_list,
                title,
                [list_widget],
                NestedWidgetLayout::default().nested_widget_layout([NestedLayoutElement(
                    Constraint::Percentage(100),
                    LayoutElement::WidgetIndex(0),
                )]),
            ),
            popup: popup_widget(tx),
        }
    }
}
