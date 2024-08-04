#![windows_subsystem = "windows"]

use std::path::PathBuf;

use app_dirs::{get_app_root, AppDataType, AppInfo};
use clap::Parser;
use clap_verbosity::Verbosity;
use gui_lib::ui_main;
use log::error;

const APP_INFO: AppInfo = AppInfo {
    name: "MathQuiz",
    author: "ChenXu",
};

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

    let db_path = match args.db_file {
        Some(p) => Some(p),
        None => match get_app_root(AppDataType::UserData, &APP_INFO) {
            Ok(p) => Some(p),
            Err(e) => {
                error!("Failed to get app data directory: {}", e);
                None
            }
        },
    };

    ui_main(db_path)?;

    c.wrap(Ok(()))
}
