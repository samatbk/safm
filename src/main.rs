use std::{env, io};

use crossterm::{
    execute,
    terminal::{self, EnterAlternateScreen},
};
use safm::fm::FileManager;

fn main() {
    terminal::enable_raw_mode().expect("Could not enter to raw mode");
    execute!(io::stdout(), EnterAlternateScreen).unwrap();

    let mut fm = FileManager::new(env::current_dir().unwrap());

    fm.update_entries().unwrap();

    loop {
        fm.cycle().unwrap();
    }
}
