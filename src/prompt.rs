use std::collections::HashMap;

use crate::commands::Command;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::backend::Backend;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;
use tui_textarea::{CursorMove, Input, Key};
use tui_textarea::TextArea;
use crate::app::KeyPress;

pub trait Prompt {
  fn prompt_text(&self) -> &str;
  fn on_submit(&mut self, input: &str) -> Option<Command>;
  fn on_cancel(&mut self) -> Option<Command> {
    None
  }
  fn on_complete(&mut self, input: &str) -> Vec<String> {
    Vec::new()
  }
  fn init_text(&self) -> String {
    String::new()
  }
}

struct PromptState<'a> {
  pub prompt: Box<dyn Prompt>,
  textarea: TextArea<'a>,
  history: Vec<String>,
  hist_index: usize,
}
//pub fn input(&mut self, input: impl Into<Input>) -> bool
// self.textarea.input(input);
impl Into<Input> for KeyPress {
  fn into(self) -> Input {
    self.to_input()
  }
}
impl<'a> PromptState<'a> {
  pub fn new(prompt: Box<dyn Prompt>, mut history: Vec<String>) -> Self {
    history.insert(0, String::new());
    let mut textarea = TextArea::new(vec![prompt.init_text()]);
    textarea.move_cursor(CursorMove::End);
    PromptState {
      textarea,
      prompt,
      history,
      hist_index: 0,
    }
  }
  /// Returns true if the prompt should be exited
  pub fn on_key(&mut self, key: KeyPress) -> (bool, Option<Command>) {
    match key {
      KeyPress(KeyCode::Char('\n'),_) => (true, self.submit()),
      KeyPress(KeyCode::Up, _) => {
        self.walk_history(1);
        (false, None)
      }
      KeyPress(KeyCode::Down, _) => {
        self.walk_history(-1);
        (false, None)
      }
      KeyPress(KeyCode::Esc, _) => (true, self.cancel()),
      input => {
        self.textarea.input(input);
        self.history[0] = self.textarea.lines()[0].clone();
        (false, None)
      }
    }
  }

  fn walk_history(&mut self, i: isize) {
    self.hist_index = self.hist_index.saturating_add_signed(i);
    self.hist_index = self.hist_index.clamp(0, self.history.len() - 1);
    self.textarea = TextArea::new(vec![self.history[self.hist_index].clone()]);
    self.textarea.move_cursor(CursorMove::End);
  }

  pub fn submit(&mut self) -> Option<Command> {
    let cmd = self.prompt.on_submit(self.textarea.lines()[0].as_str());
    self.history[0] = self.textarea.lines()[0].clone();
    self.textarea = TextArea::default();
    cmd
  }
  
  pub fn cancel(&mut self) -> Option<Command> {
    let cmd = self.prompt.on_cancel();
    self.history.remove(0);
    self.textarea = TextArea::default();
    cmd
  }

  pub fn draw(&mut self, f: &mut Frame, rect: Rect) {
    let widget = self.textarea.widget();
    let prompt = self.prompt.prompt_text();
    let text = vec![Line::from(vec![Span::raw(prompt)])];
    let input = Paragraph::new(text);
    let area1 = Rect {
      width: prompt.len() as u16,
      ..rect
    };
    let area2 = Rect {
      x: rect.x + prompt.len() as u16,
      width: rect.width - prompt.len() as u16,
      ..rect
    };
    f.render_widget(input, area1);
    f.render_widget(widget, area2);
  }
}

pub struct InfoBox {
  info_msg: String,
}

impl InfoBox {
  pub fn new() -> InfoBox {
    InfoBox {
      info_msg: String::new(),
    }
  }
  pub fn info(&mut self, msg: &str) {
    self.info_msg = String::from(msg);
  }
  pub fn error(&mut self, msg: &str) {
    self.info_msg = String::from(msg);
  }
  pub fn clear(&mut self) {
    self.info_msg.clear();
  }
}

pub struct StatusLine<'a> {
  histories: HashMap<String, Vec<String>>,
  prompt_state: Option<PromptState<'a>>,
  pub info: InfoBox,
}

impl<'a> StatusLine<'a> {
  pub fn new() -> StatusLine<'a> {
    StatusLine {
      histories: Default::default(),
      prompt_state: None,
      info: InfoBox::new(),
    }
  }
  /// Whether the statusline should get key events
  pub fn has_focus(&self) -> bool {
    self.prompt_state.is_some()
  }

  /// Handle a key
  /// Return true if the tree should be updated
  pub fn on_key(&mut self, key: KeyPress) -> (bool, Option<Command>) {
    if let Some(p) = &mut self.prompt_state {
      let (exit, cmd) = p.on_key(key);
      if exit {
        let p = self.prompt_state.take().unwrap();
        let mut hist = p.history;
        hist.dedup();
        self.histories.insert(p.prompt.prompt_text().into(), hist);
      }
      return (exit, cmd);
    }
    (false, None)
  }

  pub fn prompt(&mut self, prompt: Box<dyn Prompt>) {
    self.info.clear();
    let hist = self
      .histories
      .remove(prompt.prompt_text())
      .unwrap_or_default();
    self.prompt_state = Some(PromptState::new(prompt, hist));
  }

  pub fn draw(&mut self, f: &mut Frame, rect: Rect) {
    if let Some(prompt) = &mut self.prompt_state {
      prompt.draw(f, rect);
    } else {
      let text = vec![Line::from(vec![Span::raw(self.info.info_msg.as_str())])];
      let input = Paragraph::new(text);
      f.render_widget(input, rect);
    }
  }
  
  pub fn cancel_prompt(&mut self) -> Option<Command> {
    if let Some(p) = &mut self.prompt_state {
      let res = p.cancel();
      self.prompt_state = None;
      return res
    }
    None
  }
}
