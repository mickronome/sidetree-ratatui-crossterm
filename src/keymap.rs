use crate::Command;
use combine::parser::char::char;
use combine::parser::char::letter;
use combine::parser::char::string;
use combine::*;
use std::collections::HashMap;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::event::KeyCode::KeypadBegin;
use crate::app::{KeyPress};


pub struct KeyMap {
  keys: HashMap<KeyPress, Command>,
}
impl KeyMap {
  pub fn new() -> KeyMap {
    KeyMap {
      keys: HashMap::new(),
    }
  }

  pub fn add_mapping(&mut self, k: KeyPress, c: Command) {
    self.keys.insert(k, c);
  }

  pub fn get_mapping(&self, k: KeyPress) -> Option<Command> {
    self.keys.get(&k).cloned()
  }
}

pub fn parse_key(input: &str) -> Result<KeyPress, easy::ParseError<&str>> {
  let char_key = || {
    many1(none_of(">".chars())).and_then(|word: String| match word.as_str() {
      "return" => Ok('\n'),
      "ret" => Ok('\n'),
      "semicolon" => Ok(';'),
      "gt" => Ok('>'),
      "lt" => Ok('<'),
      "percent" => Ok('%'),
      "space" => Ok(' '),
      "tab" => Ok('\t'),
      c if c.len() == 1 => Ok(c.chars().next().unwrap()),
      &_ => Err(error::UnexpectedParse::Unexpected),
    }).map(|c| KeyPress(KeyCode::Char(c),KeyModifiers::NONE))
  };
 let modifier = || {
    optional(choice!(
      attempt(string("a-").map(|_| -> fn() -> KeyPress {|| {KeyPress(KeyCode::Esc,KeyModifiers::ALT) }})),
      attempt(string("c-").map(|_| -> fn() -> KeyPress {|| {KeyPress(KeyCode::Esc,KeyModifiers::CONTROL) }}))
    ))
    .map(|x| x.unwrap_or(|| {KeyPress(KeyCode::Esc,KeyModifiers::NONE)}))
  };
  let non_mod = || {
    many1(letter()).and_then(|word: String| match word.as_str() {
      "esc" => Ok(KeyCode::Esc),
      "backtab" => Ok(KeyCode::BackTab),
      "backspace" => Ok(KeyCode::Backspace),
      "del" => Ok(KeyCode::Delete),
      "home" => Ok(KeyCode::Home),
      "end" => Ok(KeyCode::End),
      "up" => Ok(KeyCode::Up),
      "down" => Ok(KeyCode::Down),
      "left" => Ok(KeyCode::Left),
      "right" => Ok(KeyCode::Right),
      "insert" => Ok(KeyCode::Insert),
      "pageup" => Ok(KeyCode::PageUp),
      "pagedown" => Ok(KeyCode::PageDown),
      &_ => Err(error::UnexpectedParse::Unexpected),
    }).map(|kc| KeyPress(kc,KeyModifiers::NONE))
  };
  let short = || char_key().map(|c|c);
  let long = || {
    between(
      char('<'),
      char('>'),
      attempt(
        modifier()
            .and(char_key())
            .map(
              |(f, c)| f().charize(c)))
          .or(non_mod()),
    )
  };
  let parser = long().or(short());

  parser.skip(eof()).easy_parse(input).map(|(k, _)| k)
}

#[cfg(test)]
mod tests {
  use crate::keymap::parse_key;

  use crossterm::event::{KeyCode, KeyEvent,KeyModifiers};
  use crate::app::{AltPressed, KeyPress};

  #[test]
  fn key_parsing() {
    assert_eq!(parse_key("a"), Ok(KeyCode::Char('a')));
    assert_eq!(parse_key("<a>"), Ok(KeyCode::Char('a')));
    assert_eq!(parse_key("<a-a>"), Ok(KeyPress{ code:KeyCode::Char('a'),alt:AltPressed(true),,..KeyPress::default()}));
    assert_eq!(parse_key("<c-b>"), Ok(KeyCode::Ctrl('b')));
    assert_eq!(parse_key("<return>"), Ok(Key::Char('\n')));
    assert_eq!(parse_key("<esc>"), Ok(KeyCode::Esc));
  }
}
