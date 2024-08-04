#![windows_subsystem = "windows"]

use std::{path::PathBuf, rc::Rc};

use clap::Parser;
use clap_verbosity::Verbosity;
use gui_lib::ui_main;
use log::{debug, info};
use once_cell::sync::OnceCell;
use slint::{SharedString, StandardListViewItem, VecModel, Weak};

slint::include_modules!();
struct ConsoleHolder;

impl ConsoleHolder {
    /// Attach console from parent process.
    pub fn new() -> Self {
        #[cfg(windows)]
        unsafe {
            winapi::um::wincon::AttachConsole(0xFFFFFFFF);
        }
        Self
    }

    pub fn wrap<T>(self, t: T) -> T {
        t
    }
}

#[derive(Parser, Debug)]
struct Args {
    /// Database file name.
    #[arg(short, long)]
    db_file: Option<PathBuf>,

    /// Verbosity level.
    #[command(flatten)]
    verbose: Verbosity,
}

fn main() -> anyhow::Result<()> {
    let c = ConsoleHolder::new();

    let args = Args::parse();

    env_logger::builder()
        .filter_level(args.verbose.log_level_filter())
        .init();

    ui_main(p(args.db_file))?;

    c.wrap(Ok(()))
}
