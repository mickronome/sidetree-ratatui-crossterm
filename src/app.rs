use crate::cache::Cache;
use crate::commands::parse_cmds;
use crate::commands::read_config_file;
use crate::commands::Command;
use crate::config::Config;
use crate::file_tree::{FileTree, FileTreeState};
use crate::keymap::KeyMap;
use crate::prompt::Prompt;
use crate::prompt::StatusLine;
use crossterm::event::{KeyCode, KeyModifiers, KeyEvent, MouseEvent, MouseButton, MouseEventKind};
use std::path::{Path, PathBuf};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::Frame;
use tui_textarea::{Input, Key};
use crate::Opts;


pub struct App<'a> {
  pub opts:&'a Opts,
  pub enhanced_graphics: bool,
  pub config: Config,
  pub tree: FileTreeState,
  pub exit: bool,
  pub statusline: StatusLine<'a>,
  pub keymap: KeyMap,
}


#[derive(Debug, Clone, PartialEq,Eq,Hash,Copy)]
pub struct KeyPress(pub KeyCode,pub KeyModifiers);

impl KeyPress {
  //pub fn modify(&self,modifier:KeyPress)->KeyPress {
  //  KeyPress(self.0,modifier.1)
  //}
  pub fn charize(&self,modifier:KeyPress)->KeyPress {
    KeyPress(modifier.0,self.1)
  }


  pub fn has_modifier(&self,km:KeyModifiers)->bool {
    self.1 & km != KeyModifiers::NONE
  }

  pub fn has_alt(&self)->bool {
    self.has_modifier(KeyModifiers::ALT)
  }
  pub fn has_control(&self)->bool {
    self.has_modifier(KeyModifiers::CONTROL)
  }

  pub fn to_input(&self) -> Input {
    let key = match self.0 {
      KeyCode::Backspace => {Key::Backspace}
      KeyCode::Enter => {Key::Enter}
      KeyCode::Left => {Key::Left}
      KeyCode::Right => {Key::Right}
      KeyCode::Up => {Key::Up}
      KeyCode::Down => {Key::Down}
      KeyCode::Home => {Key::Home}
      KeyCode::End => {Key::End}
      KeyCode::PageUp => {Key::PageUp}
      KeyCode::PageDown => {Key::PageDown}
      KeyCode::Tab => {Key::Tab}
      KeyCode::BackTab => {Key::Null}
      KeyCode::Delete => {Key::Delete}
      KeyCode::Insert => {Key::Null}
      KeyCode::F(t) => {Key::F(t)}
      KeyCode::Char(t) => {Key::Char(t)}
      KeyCode::Null => {Key::Null}
      KeyCode::Esc => {Key::Esc}
      KeyCode::CapsLock => {Key::Null}
      KeyCode::ScrollLock => {Key::Null}
      KeyCode::NumLock => {Key::Null}
      KeyCode::PrintScreen => {Key::Null}
      KeyCode::Pause => {Key::Null}
      KeyCode::Menu => {Key::Null}
      KeyCode::KeypadBegin => {Key::Null}
      KeyCode::Media(_) => {Key::Null}
      KeyCode::Modifier(_) => {Key::Null}
    };
    Input{key:key,
      alt:self.has_alt(),
      ctrl:self.has_control(),
      shift:self.has_modifier(KeyModifiers::SHIFT)}
  }
}
impl From<KeyEvent> for KeyPress {
   fn from(ke : KeyEvent) -> KeyPress{
  KeyPress(ke.code,ke.modifiers)
  }
}
impl From<char> for KeyPress {
  fn from(c: char) -> KeyPress {
    KeyPress(KeyCode::Char(c), KeyModifiers::NONE)
  }
}

impl From<KeyCode> for KeyPress {
  fn from(kc: KeyCode) -> KeyPress {
    KeyPress(kc, KeyModifiers::NONE)
  }
}
impl<'a> App<'a> {
  pub fn new(opts:&'a Opts, cache: Cache, enhanced_graphics:bool) -> App<'a> {
    let mut res = App {
      opts,
      enhanced_graphics,
      config: Config::default(),
      tree: FileTreeState::new(PathBuf::from(".")),
      exit: false,
      statusline: StatusLine::new(),
      keymap: KeyMap::new(),
    };
    res.read_cache(cache);
    res.tree.update(&res.config);
    res
  }
}



impl<'a> App<'a> {
  pub fn draw(&mut self, f: &mut Frame) {
    let chunks = Layout::default()
      .direction(Direction::Vertical)
      .constraints([Constraint::Min(0), Constraint::Length(1)].as_ref())
      .split(f.size());

    f.render_stateful_widget(FileTree::new(&self.config), chunks[0], &mut self.tree);
    self.statusline.draw(f, chunks[1]);
  }

  pub fn read_cache(&mut self, cache: Cache) {
    self.tree.extend_expanded_paths(cache.expanded_paths);
    self.tree.update(&self.config);
    self.tree.select_path(&cache.selected_path);
  }

  pub fn get_cache(&self) -> Cache {
    Cache {
      expanded_paths: self.tree.expanded_paths.clone(),
      selected_path: self.tree.entry().path.clone(),
    }
  }

  pub fn update(&mut self) {
    self.tree.update(&self.config);
  }

  pub fn tick(&mut self) {
    self.update();
  }

  pub fn on_mouse(&mut self, me: MouseEvent) -> Option<()> {
    if self.statusline.has_focus() {
      return Some(());
    }


      match me.kind {

        MouseEventKind::Down(MouseButton::Left) | MouseEventKind::Down(MouseButton::Right) => {
          let line = (me.row - 1) as usize;
          if self.tree.selected_idx() == Some(line) {
            let entry = self.tree.entry().clone();
            if entry.is_dir {
              self.tree.toggle_expanded(&entry.path);
            } else {
              self.run_command(&Command::Open(None))
            }
          } else {
            self.tree.select_nth(line);
          }
        }
        MouseEventKind::ScrollDown => {
          self.tree.select_next();
        }
        MouseEventKind::ScrollUp => {
          self.tree.select_prev();
        }
        _ => {}

    };
    Some(())
  }

  
  pub fn on_key(&mut self, _k:KeyEvent ) -> Option<()> {
    let k = KeyPress::from(_k);
    if self.statusline.has_focus() {
      let (update, cmd) = self.statusline.on_key(k);
      if let Some(cmd) = cmd {
        self.run_command(&cmd);
      }
      if update {
        self.update();
      }
      return Some(());
    }
    self.keymap.get_mapping(
      k.clone())
        .and_then(|cmd| {
          self.run_command(&cmd);
                      return Some(());
        });

    match k {
      KeyPress(KeyCode::Char('q'),_) => {
        self.exit = true;
      }
      KeyPress(KeyCode::Char('j') | KeyCode::Down,_) => {
        self.tree.select_next();
      }
      KeyPress(KeyCode::Char('k') | KeyCode::Up,_ ) => {
        self.tree.select_prev();
      }
      KeyPress(KeyCode::Char('\n'), _,) => {
        let entry = self.tree.entry().clone();
        if entry.is_dir {
          self.tree.toggle_expanded(&entry.path);
        } else {
          self.run_command(&Command::Open(None))
        }
      }
      KeyPress(KeyCode::Char('l'), m) if (m & KeyModifiers::ALT) == KeyModifiers::NONE => {
        self.run_command(&Command::Cd(None));
      }

      KeyPress(KeyCode::Char('l') | KeyCode::Right, _) => {
        let entry = self.tree.entry().clone();
        if entry.is_dir {
          if !entry.is_expanded() {
            self.tree.expand(&entry.path);
          } else {
            self.tree.select_next();
          }
        }
      }
      KeyPress(KeyCode::Char('h') | KeyCode::Left, _) => {
        let entry = self.tree.entry().clone();
        if entry.is_expanded() {
          self.tree.collapse(&entry.path);
        } else {
          self.tree.select_up();
        }
      }
      KeyPress(KeyCode::Char('!'), _) => {
        self.statusline.prompt(Box::new(ShellPrompt {}));
      }
      KeyPress(KeyCode::Char(':'), _) => {
        self.statusline.prompt(Box::new(CmdPrompt {}));
      }
      KeyPress(KeyCode::Char('.'), _) => {
        self.config.show_hidden = !self.config.show_hidden;
      }
      _ => {}
    }
    Some(())
  }

  pub fn run_commands(&mut self, cmds: &Vec<Command>) {
    for c in cmds {
      self.run_command(c);
    }
  }

  pub fn run_command(&mut self, cmd: &Command) {
    use Command::*;
    match cmd {
      Quit => {
        self.quit();
      }
      Shell(cmd) => {
        self.run_shell(cmd.as_str());
      }
      Open(path) => {
        let cmd = self.config.open_cmd.clone();
        let path = path.as_ref().unwrap_or_else(|| &self.tree.entry().path);
        let _path = path.clone();
        self.run_shell(cmd.as_str());
        if self.config.quit_on_open {
          self.quit();
        }
      }
      CmdStr(cmd) => match parse_cmds(cmd) {
        Ok(cmds) => self.run_commands(&cmds),
        Err(msg) => self.error(msg.as_str()),
      },
      Set(opt, val) => {
        if let Err(e) = self.config.set_opt(opt, val) {
          self.statusline.info.error(e.as_str());
        }
      }
      Echo(msg) => {
        self.statusline.info.info(msg.as_str());
      }
      Cd(path) => {
        let path = path.as_ref().unwrap_or_else(|| &self.tree.entry().path);
        let path = path.clone();
        match std::env::set_current_dir(path.as_path()) {
          Ok(()) => self
            .tree
            .change_root(&self.config, std::env::current_dir().unwrap()),
          Err(err) => self.error(err.to_string().as_str()),
        }
      }
      MapKey(key, cmd) => {
        self.keymap.add_mapping(*key, (**cmd).clone());
      }
      Rename(name) => {
        if let Some(name) = name {
          let src = &self.tree.entry().path;
          let mut dst = src.clone();
          dst.set_file_name(name);
          // TODO: Error handling
          if !dst.exists() {
            std::fs::rename(src, dst).unwrap();
          }
        } else {
          self.statusline.prompt(Box::new(RenamePrompt {
            old_name: self
              .tree
              .entry()
              .path
              .file_name()
              .unwrap()
              .to_string_lossy()
              .into(),
          }));
        }
      }
      NewFile(name) => {
        if let Some(name) = name {
          let mut path = self.tree.current_dir();
          path.push(name);
          // TODO: Error handling
          if !path.exists() {
            if name.ends_with('/') {
              std::fs::create_dir_all(path).unwrap();
            } else {
              std::fs::write(path, "").unwrap();
            }
          }
        } else {
          self.statusline.prompt(Box::new(NewFilePrompt {}));
        }
      }
      NewDir(name) => {
        if let Some(name) = name {
          let mut path = self.tree.current_dir();
          path.push(name);
          // TODO: Error handling
          if !path.exists() {
            std::fs::create_dir_all(path).unwrap();
          }
        } else {
          self.statusline.prompt(Box::new(NewDirPrompt {}));
        }
      }

      Delete { prompt } => {
        if !prompt {
          let path = &self.tree.entry().path;
          // TODO: Error handling
          if path.is_dir() {
            std::fs::remove_dir_all(path).unwrap();
          } else {
            std::fs::remove_file(path).unwrap();
          }
        } else {
          self.statusline.prompt(Box::new(DeletePrompt {}));
        }
      }
    }
    self.update();
  }
  pub fn error(&mut self, msg: &str) {
    self.statusline.info.error(msg)
  }
  fn quit(&mut self) {
    self.exit = true;
  }

  pub fn run_script_file(&mut self, path: &Path) -> Result<(), String> {
    let cmds = read_config_file(path)?;
    self.run_commands(&cmds);
    Ok(())
  }

  fn run_shell(&mut self, cmd: &str) {
    let output = std::process::Command::new("sh")
      .arg("-c")
      .arg(cmd)
      .arg("--")
      .arg(self.tree.entry().path.to_str().unwrap_or(""))
      .env(
        "sidetree_root",
        self.tree.root_entry.path.to_str().unwrap_or(""),
      )
      .env(
        "sidetree_entry",
        self.tree.entry().path.to_str().unwrap_or(""),
      )
      .env(
        "sidetree_dir",
        self.tree.current_dir().to_str().unwrap_or(""),
      )
      .output();
    match output {
      Err(err) => {
        self.statusline.info.error(&err.to_string());
      }
      Ok(output) => {
        if !output.status.success() {
          self
            .statusline
            .info
            .error(format!("Command failed with {}", output.status).as_str())
        }
      }
    }
  }
}

pub struct ShellPrompt {}

impl Prompt for ShellPrompt {
  fn prompt_text(&self) -> &str {
    "!"
  }
  fn on_submit(&mut self, text: &str) -> Option<Command> {
    Some(Command::Shell(text.to_string()))
  }
  fn on_cancel(&mut self) -> Option<Command> {
    None
  }
}

pub struct CmdPrompt {}

impl Prompt for CmdPrompt {
  fn prompt_text(&self) -> &str {
    ":"
  }
  fn on_submit(&mut self, text: &str) -> Option<Command> {
    Some(Command::CmdStr(text.to_string()))
  }
}

pub struct RenamePrompt {
  old_name: String,
}

impl Prompt for RenamePrompt {
  fn prompt_text(&self) -> &str {
    "Rename>"
  }

  fn on_submit(&mut self, input: &str) -> Option<Command> {
    Some(Command::Rename(Some(input.into())))
  }

  fn init_text(&self) -> String {
    self.old_name.clone()
  }
}

pub struct NewFilePrompt {}

impl Prompt for NewFilePrompt {
  fn prompt_text(&self) -> &str {
    "mk>"
  }

  fn on_submit(&mut self, input: &str) -> Option<Command> {
    Some(Command::NewFile(Some(input.into())))
  }
}

pub struct NewDirPrompt {}

impl Prompt for NewDirPrompt {
  fn prompt_text(&self) -> &str {
    "New dir>"
  }

  fn on_submit(&mut self, input: &str) -> Option<Command> {
    Some(Command::NewDir(Some(input.into())))
  }
}

pub struct DeletePrompt {}

impl Prompt for DeletePrompt {
  fn prompt_text(&self) -> &str {
    "delete? [y/N]>"
  }

  fn on_submit(&mut self, input: &str) -> Option<Command> {
    if input == "y" || input == "Y" {
      Some(Command::Delete { prompt: false })
    } else {
      None
    }
  }
}
