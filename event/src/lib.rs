pub mod input;
pub mod tick;

pub mod kubernetes;

mod util;

use crate::kubernetes::Kube;
use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
pub use kube as kube_rs;

#[derive(PartialEq, Clone, Copy)]
pub enum UserEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
}

impl UserEvent {
    pub fn from_key(code: KeyCode) -> Self {
        UserEvent::Key(KeyEvent::from(code))
    }
}

impl From<char> for UserEvent {
    fn from(c: char) -> Self {
        UserEvent::Key(KeyEvent::from(KeyCode::Char(c)))
    }
}
impl From<KeyCode> for UserEvent {
    fn from(code: KeyCode) -> Self {
        UserEvent::Key(KeyEvent::from(code))
    }
}
pub enum Event {
    Kube(Kube),
    User(UserEvent),
    Tick,
}
