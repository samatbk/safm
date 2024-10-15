use anyhow::Result;
use crossterm::{
    queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use std::{fs::Metadata, io::Cursor};

#[derive(Clone)]
pub struct Entry {
    pub name: String,
    pub metadata: Metadata,
}

impl Entry {
    pub fn write(&self, highlight: bool, buffer: &mut Cursor<Vec<u8>>) -> Result<()> {
        queue!(
            buffer,
            if highlight {
                SetBackgroundColor(Color::DarkMagenta)
            } else {
                SetBackgroundColor(Color::Reset)
            },
            if self.metadata.is_dir() {
                SetForegroundColor(Color::Blue)
            } else {
                SetForegroundColor(Color::Reset)
            },
            Print(format!("{}\r\n", self.name)),
            ResetColor
        )?;

        Ok(())
    }
}
