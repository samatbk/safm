use anyhow::Result;
use crossterm::{
    queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use std::{fs::Metadata, io::Cursor, os::unix::fs::MetadataExt};

#[derive(Clone)]
pub struct Entry {
    pub name: String,
    pub metadata: Metadata,
}

impl Entry {
    pub fn write(&self, highlight: bool, buffer: &mut Cursor<Vec<u8>>) -> Result<()> {
        let (divisor, symbol) = match self.metadata.size() {
            0..1_000 => (1.0, ""),
            1_000..1_000_000 => (1000.0, "kb"),
            1_000_000..1_000_000_000 => (1_000_000.0, "mb"),
            _ => (1_000_000_000.0, "gb"),
        };

        queue!(
            buffer,
            SetForegroundColor(Color::Yellow),
            if highlight {
                SetBackgroundColor(Color::DarkGrey)
            } else {
                SetBackgroundColor(Color::Reset)
            },
            Print(format!(
                "{: <8}",
                if self.metadata.is_dir() {
                    "dir".to_string()
                } else {
                    format!("{:.1}{}", self.metadata.size() as f32 / divisor, symbol)
                }
            )),
            if self.metadata.is_dir() {
                SetForegroundColor(Color::Blue)
            } else {
                SetForegroundColor(Color::Reset)
            },
            Print(format!(" {: >32}\r\n", self.name)),
            ResetColor
        )?;

        Ok(())
    }
}
