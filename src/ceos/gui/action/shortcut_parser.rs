use crate::ceos::gui::action::simple_shortcut::SimpleShortcut;
use egui::{Key, Modifiers};
use logos::Logos;

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t\n\f]+")] // Ignore whitespace
enum ShortcutToken {
    #[token("Ctrl")]
    Ctrl,

    #[token("Shift")]
    Shift,

    #[token("Alt")]
    Alt,

    #[token("Cmd")]
    #[token("⌘")]
    #[token("⊞")]
    #[token("Win")]
    Command,

    #[token("+")]
    #[token("-")]
    Separator,

    #[regex(r"[a-zA-Z0-9]+", |lex| lex.slice().to_string())]
    Identifier(String),
}

pub fn parse_shortcut(input: &str) -> Result<SimpleShortcut, String> {
    let mut lexer = ShortcutToken::lexer(input);
    let mut modifiers = Modifiers::NONE;
    let mut key = None;

    while let Some(token_res) = lexer.next() {
        let token = token_res.map_err(|_| format!("Invalid token at '{}'", lexer.slice()))?;
        match token {
            ShortcutToken::Ctrl => modifiers.ctrl = true,
            ShortcutToken::Shift => modifiers.shift = true,
            ShortcutToken::Alt => modifiers.alt = true,
            ShortcutToken::Command => modifiers.command = true,
            ShortcutToken::Separator => {}
            ShortcutToken::Identifier(s) => {
                if let Some(k) = string_to_key(&s) {
                    if key.is_some() {
                        return Err(format!(
                            "Multiple keys specified: {:?} and {:?}",
                            key.unwrap(),
                            k
                        ));
                    }
                    key = Some(k);
                } else {
                    // Maybe it's a modifier written differently?
                    match s.to_lowercase().as_str() {
                        "ctrl" | "control" => modifiers.ctrl = true,
                        "shift" => modifiers.shift = true,
                        "alt" => modifiers.alt = true,
                        "command" | "cmd" | "super" | "win" => modifiers.command = true,
                        _ => return Err(format!("Unknown key or modifier: {}", s)),
                    }
                }
            }
        }
    }

    let key = key.ok_or_else(|| "No key specified in shortcut".to_string())?;
    Ok(SimpleShortcut::new(modifiers, key))
}

fn string_to_key(s: &str) -> Option<Key> {
    match s.to_lowercase().as_str() {
        "a" => Some(Key::A),
        "b" => Some(Key::B),
        "c" => Some(Key::C),
        "d" => Some(Key::D),
        "e" => Some(Key::E),
        "f" => Some(Key::F),
        "g" => Some(Key::G),
        "h" => Some(Key::H),
        "i" => Some(Key::I),
        "j" => Some(Key::J),
        "k" => Some(Key::K),
        "l" => Some(Key::L),
        "m" => Some(Key::M),
        "n" => Some(Key::N),
        "o" => Some(Key::O),
        "p" => Some(Key::P),
        "q" => Some(Key::Q),
        "r" => Some(Key::R),
        "s" => Some(Key::S),
        "t" => Some(Key::T),
        "u" => Some(Key::U),
        "v" => Some(Key::V),
        "w" => Some(Key::W),
        "x" => Some(Key::X),
        "y" => Some(Key::Y),
        "z" => Some(Key::Z),
        "0" => Some(Key::Num0),
        "1" => Some(Key::Num1),
        "2" => Some(Key::Num2),
        "3" => Some(Key::Num3),
        "4" => Some(Key::Num4),
        "5" => Some(Key::Num5),
        "6" => Some(Key::Num6),
        "7" => Some(Key::Num7),
        "8" => Some(Key::Num8),
        "9" => Some(Key::Num9),
        "f1" => Some(Key::F1),
        "f2" => Some(Key::F2),
        "f3" => Some(Key::F3),
        "f4" => Some(Key::F4),
        "f5" => Some(Key::F5),
        "f6" => Some(Key::F6),
        "f7" => Some(Key::F7),
        "f8" => Some(Key::F8),
        "f9" => Some(Key::F9),
        "f10" => Some(Key::F10),
        "f11" => Some(Key::F11),
        "f12" => Some(Key::F12),
        "f13" => Some(Key::F13),
        "f14" => Some(Key::F14),
        "f15" => Some(Key::F15),
        "f16" => Some(Key::F16),
        "f17" => Some(Key::F17),
        "f18" => Some(Key::F18),
        "f19" => Some(Key::F19),
        "f20" => Some(Key::F20),
        "arrowdown" | "down" => Some(Key::ArrowDown),
        "arrowleft" | "left" => Some(Key::ArrowLeft),
        "arrowright" | "right" => Some(Key::ArrowRight),
        "arrowup" | "up" => Some(Key::ArrowUp),
        "escape" | "esc" => Some(Key::Escape),
        "tab" => Some(Key::Tab),
        "backspace" => Some(Key::Backspace),
        "enter" | "return" => Some(Key::Enter),
        "space" => Some(Key::Space),
        "insert" => Some(Key::Insert),
        "delete" | "del" => Some(Key::Delete),
        "home" => Some(Key::Home),
        "end" => Some(Key::End),
        "pageup" => Some(Key::PageUp),
        "pagedown" => Some(Key::PageDown),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let s = parse_shortcut("S").unwrap();
        assert_eq!(s, SimpleShortcut::new(Modifiers::NONE, Key::S));
    }

    #[test]
    fn test_parse_ctrl_s() {
        let s = parse_shortcut("Ctrl+S").unwrap();
        assert_eq!(s, SimpleShortcut::new(Modifiers::CTRL, Key::S));
    }

    #[test]
    fn test_parse_complex() {
        let s = parse_shortcut("Ctrl+Shift+F7").unwrap();
        assert_eq!(
            s,
            SimpleShortcut::new(Modifiers::CTRL | Modifiers::SHIFT, Key::F7)
        );
    }

    #[test]
    fn test_parse_lowercase() {
        let s = parse_shortcut("ctrl+shift+f7").unwrap();
        assert_eq!(
            s,
            SimpleShortcut::new(Modifiers::CTRL | Modifiers::SHIFT, Key::F7)
        );
    }

    #[test]
    fn test_parse_spaces() {
        let s = parse_shortcut("Ctrl + Shift + F7").unwrap();
        assert_eq!(
            s,
            SimpleShortcut::new(Modifiers::CTRL | Modifiers::SHIFT, Key::F7)
        );
    }

    #[test]
    fn test_parse_cmd() {
        let s = parse_shortcut("Cmd+Z").unwrap();
        assert_eq!(s, SimpleShortcut::new(Modifiers::COMMAND, Key::Z));
    }
}
