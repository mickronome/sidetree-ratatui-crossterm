mod app;
mod cache;
mod commands;
mod config;
mod file_tree;
mod icons;
mod keymap;
mod prompt;
mod util;

use crate::commands::Command;
use crate::{app::App, cache::Cache};
use std::{fs::File, path::PathBuf};

use clap::Parser;
use commands::parse_cmds;
use std::{
  error::Error,
  io,
  time::{Duration, Instant},
};
use ratatui::backend::{CrosstermBackend};
use ratatui::Terminal;

use crossterm::{
  event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
  execute,
  terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

//use crate::util::event::{Event, Events};

extern crate combine;

#[derive(Parser)]
#[clap(
  version = env!("CARGO_PKG_VERSION"),
  author = env!("CARGO_PKG_AUTHORS"),
)]
/// An interactive file tree meant to be used as a side panel for terminal text editors

struct Opts {
  /// The base directory to open sidetree to
  #[clap(default_value = ".")]
  directory: PathBuf,

  /// Set a config file to use. By default uses $XDG_CONFIG_DIR/sidetree/sidetreerc
  #[clap(short, long)]
  config: Option<PathBuf>,

  /// Unless this is set, expanded paths and current selection will be saved in
  /// $XDG_CACHE_DIR/sidetree/sidetreecache.toml
  #[clap(long)]
  no_cache: bool,

  /// Preselect a path. Will expand all directories up to the path
  #[clap(short, long)]
  select: Option<PathBuf>,

  /// Commands to run on startup
  #[clap(short, long)]
  exec: Option<String>,
}

const DEFAULT_CONFIG: &str = include_str!("../sidetreerc");

fn default_conf_file() -> PathBuf {
  let xdg = xdg::BaseDirectories::with_prefix("sidetree").unwrap();
  let conf_file = xdg
    .place_config_file("sidetreerc")
    .expect("Cannot create config directory");
  if !conf_file.exists() {
    File::create(&conf_file).expect("Cannot create config file");
    std::fs::write(&conf_file, DEFAULT_CONFIG).expect("Couldn't write default config file");
  }
  conf_file
}

pub fn run(opts: &Opts,cache: Cache,tick_rate: Duration, enhanced_graphics: bool) -> Result<(), Box<dyn Error>> {
  // setup terminal
  enable_raw_mode()?;
  let mut stdout = io::stdout();

  crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

  let backend = CrosstermBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;

  // create app and run it
  let mut app = App::new(opts,cache,enhanced_graphics);
  let conf_file = opts.config.clone().unwrap_or_else(default_conf_file);

  app.run_script_file(&conf_file)?;
  if opts.exec.is_some() {
    app.run_commands(&parse_cmds(&opts.exec.clone().unwrap())?)
  }

  app.tree.change_root(&app.config, opts.directory.clone());

  if let Some(path) = opts.select.clone() {
    app.tree.expand_to_path(&path);
    app.tree.update(&app.config);
    app.tree.select_path(&path);
  }

  let res = run_app(&mut terminal, app, tick_rate);

  // restore terminal
  disable_raw_mode()?;
  execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
  terminal.show_cursor()?;

  if let Err(err) = res {
    println!("{err:?}");
  }

  Ok(())
}
fn run_app<B: Backend>(
  terminal: &mut Terminal<B>,
  mut app: App,
  tick_rate: Duration,
) -> io::Result<()> {
  let mut last_tick = Instant::now();
  loop {
    terminal.draw(|f| app.draw(f))?;

    let timeout = tick_rate.saturating_sub(last_tick.elapsed());
    if crossterm::event::poll(timeout)? {
      if let Event::Key(key) = event::read()? {
        app.on_key(key);


      }
    }
    if last_tick.elapsed() >= tick_rate {
      app.tick();
      last_tick = Instant::now();
    }
    if app.exit {
      if !app.opts.no_cache {
        app.get_cache().write_file(&Cache::default_file_path())
      }
      return Ok(());
    }
  }
}
fn main() -> Result<(), Box<dyn Error>> {
  let opts = Opts::parse();

  // Terminal initialization
  let tick_rate = Duration::from_millis(250);

  let cache = if !opts.no_cache {
    Cache::from_file(&Cache::default_file_path()).expect("Failed to read cache file")
  } else {
    Cache::default()
  };

    run(&opts, cache, tick_rate, true)?;


  Ok(())
}
