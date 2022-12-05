mod app;
mod apply_colors;
mod color;
mod scheme;
mod utils;

use std::ffi::OsStr;
use std::{env, path::Path};

use app::{App, AppConfig};
use utils::Result;

fn main() {
    if let Err(err) = run() {
        println!("Error: {}", err);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args: Vec<_> = env::args().collect();

    if args.len() != 2 {
        // Print help and exit
        let bin_name = args
            .get(0)
            .map(Path::new)
            .and_then(Path::file_name)
            .and_then(OsStr::to_str)
            .unwrap_or("base16-colorizer");
        println!("USAGE: {bin_name} [--init|scheme-name]");
    } else if args[1] == "--init" {
        // Set up up config directory and exit
        App::init_config_dir()?;
    } else {
        // Entry point to applications
        App::try_from_config(AppConfig {
            scheme: args[1].to_owned(),
        })?
        .run()?;
    }

    Ok(())
}
