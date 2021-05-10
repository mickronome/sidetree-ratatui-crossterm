use crate::App;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
  pub show_hidden: bool,
  pub open_cmd: String,
  pub quit_on_open: bool,
  pub file_icons: bool,
}

impl Default for Config {
  fn default() -> Config {
    Config {
      show_hidden: false,
      open_cmd: String::from("kcr edit \"$1\"; kcr send focus"),
      quit_on_open: false,
      file_icons: false,
    }
  }
}

impl Config {
  pub fn set_opt(&mut self, opt: &str, val: &str) -> Result<(), String> {
    match opt {
      "open_cmd" => {
        self.open_cmd = val.to_string();
        Ok(())
      }
      "show_hidden" => {
        self.show_hidden = Self::parse_opt(val)?;
        Ok(())
      }
      "quit_on_open" => {
        self.quit_on_open = Self::parse_opt(val)?;
        Ok(())
      }
      "file_icons" => {
        self.file_icons = Self::parse_opt(val)?;
        Ok(())
      }
      _ => Err(format!("unknown option {}", opt)),
    }
  }

  fn parse_opt<T: std::str::FromStr>(val: &str) -> Result<T, String> {
    match val.parse::<T>() {
      Ok(res) => Ok(res),
      Err(_) => Err("Could not parse option value".to_string()),
    }
  }
}
