use crossterm::{
    cursor,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{self, Clear, ClearType},
};

use crate::entry::Entry;

use std::{
    fs::{self},
    io::{self, Cursor, Write},
    path::PathBuf,
};

use anyhow::{bail, Result};

pub struct FileManager {
    position: usize,
    position_history: Vec<usize>,

    term_height: u16,
    entries: Vec<Entry>,
    buffer: Cursor<Vec<u8>>,
    current_path: PathBuf,

    is_viewing_file: bool,
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
            is_viewing_file: false,
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

    pub fn draw_ui(&mut self) -> Result<()> {
        let mut stdout = io::stdout();

        self.buffer.get_mut().clear();

        crossterm::queue!(self.buffer, Clear(ClearType::All), cursor::MoveTo(0, 0))?;

        crossterm::queue!(
            self.buffer,
            SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::Cyan),
            Print(format_args!("{}:\r\n", self.current_path.display())),
            SetAttribute(Attribute::Reset),
            ResetColor
        )?;

        if !self.is_viewing_file {
            self.update_buffer()?;
            stdout.write_all(&self.buffer.get_ref())?;
        }

        Ok(())
    }

    pub fn write_to_buffer(&mut self, content: String) -> Result<()> {
        let mut stdout = io::stdout();

        crossterm::queue!(self.buffer, Clear(ClearType::All), cursor::MoveTo(0, 0))?;
        crossterm::execute!(self.buffer, Print(content))?;
        stdout.write_all(&self.buffer.get_ref())?;
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

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn set_position(&mut self, position: usize) {
        self.position = position;
    }

    pub fn entries(&self) -> &Vec<Entry> {
        &self.entries
    }

    pub fn entries_mut(&mut self) -> &mut Vec<Entry> {
        &mut self.entries
    }

    pub fn current_path(&self) -> &PathBuf {
        &self.current_path
    }

    pub fn current_path_mut(&mut self) -> &mut PathBuf {
        &mut self.current_path
    }

    pub fn set_current_path(&mut self, path: PathBuf) {
        self.current_path = path;
    }

    pub fn position_history(&self) -> &Vec<usize> {
        &self.position_history
    }

    pub fn position_history_mut(&mut self) -> &mut Vec<usize> {
        &mut self.position_history
    }

    pub fn buffer_mut(&mut self) -> &mut Cursor<Vec<u8>> {
        &mut self.buffer
    }

    pub fn current_entry(&self) -> &Entry {
        &self.entries[self.position]
    }

    pub fn buffer(&self) -> &Cursor<Vec<u8>> {
        &self.buffer
    }

    pub fn toggle_file_view(&mut self) {
        self.is_viewing_file = !self.is_viewing_file
    }
}
