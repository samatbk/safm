use anyhow::Result;
use std::io::Write;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{self, EnterAlternateScreen},
};
use safm::{
    commands::{
        Command, EnterToDirCommand, GoToParentDirCommand, MoveCursorDownCommand,
        MoveCursorUpCommand, QuitCommand, ShowFileCommand,
    },
    fm::FileManager,
};
use std::{cell::RefCell, collections::HashMap, env, io, rc::Rc};

fn main() -> Result<()> {
    terminal::enable_raw_mode().expect("Could not enter raw mode");
    execute!(io::stdout(), EnterAlternateScreen)?;

    let fm = Rc::new(RefCell::new(FileManager::new(env::current_dir()?)));

    let mut command_map: HashMap<KeyCode, Box<dyn Command>> = HashMap::new();
    command_map.insert(KeyCode::Char('q'), Box::new(QuitCommand));
    command_map.insert(KeyCode::Char('k'), Box::new(MoveCursorUpCommand));
    command_map.insert(KeyCode::Char('j'), Box::new(MoveCursorDownCommand));
    command_map.insert(KeyCode::Char('l'), Box::new(EnterToDirCommand));
    command_map.insert(KeyCode::Char('h'), Box::new(GoToParentDirCommand));
    command_map.insert(KeyCode::Enter, Box::new(ShowFileCommand));

    fm.borrow_mut().update_entries()?;

    loop {
        fm.borrow_mut().draw_ui()?;

        if let Event::Key(event) = event::read()? {
            if let Some(cmd) = command_map.get(&event.code) {
                cmd.execute(Rc::clone(&fm))?;
            }
        }

        io::stdout().flush()?;
    }
}
