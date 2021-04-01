mod parser;

use parser::parse;

#[derive(Debug, PartialEq, Clone)]
pub enum AnsiEscapeSequence {
    Chars,
    Escape,
    CursorUp(u16),
    CursorDown(u16),
    CursorForward(u16),
    CursorBack(u16),
    CursorNextLine(u16),
    CursorPreviousLine(u16),
    CursorHorizontalAbs(u16),
    CursorPos(u16, u16),
    EraseDisplay(u16),
    EraseLine(u16),
    ScrollUp(u16),
    ScrollDown(u16),
    HorizontalVerticalPos(u16, u16),
    SelectGraphicRendition(Vec<u8>),
    AuxPortOn,
    AuxPortOff,
    DeviceStatusReport,
    SaveCurrentCursorPos,
    RestoreSavedCursorPos,
    CursorShow,
    CursorHide,
    SetMode(u8),
    ResetMode(u8),
}

#[derive(Debug, PartialEq)]
pub struct Text<'a> {
    chars: &'a str,
    ty: AnsiEscapeSequence,
}

impl<'a> Text<'a> {
    fn new(chars: &'a str, ty: AnsiEscapeSequence) -> Self {
        Self { chars, ty }
    }
}

pub trait TextParser {
    fn ansi_parse(&self) -> TextIterator;
}

pub struct TextIterator<'a>(&'a str);

impl TextParser for str {
    fn ansi_parse(&self) -> TextIterator {
        TextIterator(self)
    }
}

impl TextParser for String {
    fn ansi_parse(&self) -> TextIterator {
        TextIterator(self)
    }
}

impl<'a> Iterator for TextIterator<'a> {
    type Item = Text<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        use AnsiEscapeSequence::*;
        let s = self.0;

        if s.is_empty() {
            return None;
        }

        let find = s.find("\x1b");
        match find {
            Some(pos) => {
                if pos == 0 {
                    let current = &s[pos..];
                    match parse(current) {
                        Ok(ret) => {
                            self.0 = ret.0;
                            let len = current.len() - ret.0.len();
                            Some(Text::new(&s[pos..len], ret.1))
                        }
                        Err(_) => Some(Text::new("\x1b", Escape)),
                    }
                } else {
                    let ret = &s[..pos];
                    self.0 = &s[pos..];
                    Some(Text::new(ret, Chars))
                }
            }
            None => {
                let temp = s;
                self.0 = "";
                Some(Text::new(temp, Chars))
            }
        }
    }
}

#[cfg(test)]
mod parse_test {
    use super::AnsiEscapeSequence::*;
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn empty() {
        assert_eq!("".ansi_parse().next(), None);
    }

    #[test]
    fn text_only() {
        assert_eq!("text".ansi_parse().next(), Some(Text::new("text", Chars)));
    }

    #[test]
    fn escape_only() {
        assert_eq!(
            "\x1b".ansi_parse().next(),
            Some(Text::new("\x1b", AnsiEscapeSequence::Escape))
        );
    }

    #[test]
    fn escape_cursor_up() {
        assert_eq!(
            "\x1b[1A".ansi_parse().next(),
            Some(Text::new("\x1b[1A", CursorUp(1)))
        );
    }

    #[test]
    fn escape_cursor_up_and_cursor_down() {
        let mut iter = "\x1b[1A\x1b[1B".ansi_parse();
        assert_eq!(iter.next(), Some(Text::new("\x1b[1A", CursorUp(1))));
        assert_eq!(iter.next(), Some(Text::new("\x1b[1B", CursorDown(1))));
    }

    #[test]
    fn escape_color() {
        assert_eq!(
            "\x1b[1;2;3;4m".ansi_parse().next(),
            Some(Text::new(
                "\x1b[1;2;3;4m",
                SelectGraphicRendition(vec![1, 2, 3, 4])
            ))
        );
    }

    #[test]
    fn text_and_cursor_up() {
        let mut iter = "text\x1b[1A".ansi_parse();
        assert_eq!(iter.next(), Some(Text::new("text", Chars)));
        assert_eq!(iter.next(), Some(Text::new("\x1b[1A", CursorUp(1))));
    }

    #[test]
    fn cursor_up_and_text() {
        let mut iter = "\x1b[1Atext".ansi_parse();
        assert_eq!(iter.next(), Some(Text::new("\x1b[1A", CursorUp(1))));
        assert_eq!(iter.next(), Some(Text::new("text", Chars)));
    }

    #[test]
    fn text_and_cursor_up_and_text() {
        let mut iter = "text\x1b[1Atext".ansi_parse();
        assert_eq!(iter.next(), Some(Text::new("text", Chars)));
        assert_eq!(iter.next(), Some(Text::new("\x1b[1A", CursorUp(1))));
        assert_eq!(iter.next(), Some(Text::new("text", Chars)));
    }

    #[test]
    fn cursor_up_text_and_cursor_down() {
        let mut iter = "\x1b[1Atext\x1b[1B".ansi_parse();
        assert_eq!(iter.next(), Some(Text::new("\x1b[1A", CursorUp(1))));
        assert_eq!(iter.next(), Some(Text::new("text", Chars)));
        assert_eq!(iter.next(), Some(Text::new("\x1b[1B", CursorDown(1))));
    }
}
