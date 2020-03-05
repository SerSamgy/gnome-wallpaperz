/// Gets path to folder with wallpapers and generates xml file
/// # Example
/// ```
/// gw path_to_folder output_file
/// ```
use std::env;
use std::process;

use gnome_wallpaperz::Config;

fn main() {
    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    if let Err(e) = gnome_wallpaperz::run(config) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }
}
