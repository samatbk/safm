use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, Clear, ClearType, LeaveAlternateScreen},
};

use crate::entry::Entry;

use std::{
    fs::{self},
    io::{self, Cursor, Write},
    path::PathBuf,
    process::exit,
};

use anyhow::{bail, Result};

pub struct FileManager {
    position: usize,
    position_history: Vec<usize>,

    term_height: u16,
    entries: Vec<Entry>,
    buffer: Cursor<Vec<u8>>,
    current_path: PathBuf,
}

impl FileManager {
    pub fn new(directory: PathBuf) -> Self {
        Self {
            position: 0,
            position_history: Vec::new(),
            entries: vec![],
            term_height: terminal::size().unwrap().1,
            buffer: Cursor::new(Vec::new()),
            current_path: directory,
        }
    }

    fn update_buffer(&mut self) -> Result<()> {
        let amount: usize = (self.term_height - 2).into();

        let start = self.position.saturating_sub(amount / 2);

        for (i, entry) in self.entries.iter().enumerate().skip(start).take(amount) {
            entry.write(i == self.position, &mut self.buffer)?;
        }
        Ok(())
    }

    fn move_cursor_up(&mut self) {
        if self.position == 0 {
            self.position = self.entries.len() - 1;
        } else {
            self.position -= 1;
        }
    }

    fn move_cursor_down(&mut self) {
        if self.position == self.entries.len() - 1 {
            self.position = 0;
        } else {
            self.position += 1;
        }
    }

    fn enter_to_dir(&mut self) {
        let entry = &self.entries[self.position];
        if !entry.metadata.is_dir() {
            return;
        }
        self.position_history.push(self.position);
        self.current_path.push(&entry.name);

        match self.update_entries() {
            Ok(_) => {
                self.position = 0;
            }
            Err(_) => {
                self.current_path.pop();
                execute!(self.buffer, SetBackgroundColor(Color::Blue)).unwrap();
            }
        }
    }

    fn goto_parent_dir(&mut self) {
        if let Some(parent_dir) = self.current_path.parent() {
            self.current_path = parent_dir.into();
        }

        self.position = match self.position_history.pop() {
            Some(history) => history,
            None => 0,
        };

        self.update_entries().unwrap();
    }

    pub fn cycle(&mut self) -> Result<()> {
        let mut stdout = io::stdout();

        self.buffer.get_mut().clear();
        queue!(self.buffer, Clear(ClearType::All), cursor::MoveTo(0, 0)).unwrap();

        queue!(
            self.buffer,
            SetForegroundColor(Color::Red),
            Print(format!("== safm == {}\r\n", self.current_path.display())),
            ResetColor
        )?;
        self.update_buffer()?;
        stdout.write_all(&self.buffer.get_ref())?;
        stdout.flush()?;

        if let Event::Key(event) = event::read().expect("Failed to read line") {
            match event {
                KeyEvent {
                    code: KeyCode::Char('q'),
                    modifiers: KeyModifiers::NONE,
                    ..
                } => {
                    execute!(io::stdout(), LeaveAlternateScreen).unwrap();
                    terminal::disable_raw_mode().expect("Could not exit to raw mode");
                    exit(0);
                }
                KeyEvent {
                    code: KeyCode::Char('j'),
                    modifiers: KeyModifiers::NONE,
                    ..
                } => {
                    if !self.entries.is_empty() {
                        self.move_cursor_down()
                    }
                }
                KeyEvent {
                    code: KeyCode::Char('k'),
                    modifiers: KeyModifiers::NONE,
                    ..
                } => {
                    if !self.entries.is_empty() {
                        self.move_cursor_up()
                    }
                }
                KeyEvent {
                    code: KeyCode::Char('l'),
                    modifiers: KeyModifiers::NONE,
                    ..
                } => self.enter_to_dir(),
                KeyEvent {
                    code: KeyCode::Char('h'),
                    modifiers: KeyModifiers::NONE,
                    ..
                } => self.goto_parent_dir(),
                _ => {}
            }
        }

        Ok(())
    }

    pub fn update_entries(&mut self) -> Result<()> {
        let mut entries = Vec::new();

        match fs::read_dir(&self.current_path) {
            Ok(result) => {
                for entry in result {
                    let entry = entry.unwrap();
                    entries.push(Entry {
                        name: entry.file_name().into_string().unwrap(),
                        metadata: entry.metadata().unwrap(),
                    });
                }
            }
            Err(error) => {
                bail!(error);
            }
        }

        self.entries = entries.clone();
        Ok(())
    }
}
