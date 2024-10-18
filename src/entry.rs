use anyhow::Result;
use crossterm::{
    queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use std::{
    fs::Metadata,
    io::Cursor,
    os::unix::fs::{MetadataExt, PermissionsExt},
};

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

        let mode = self.metadata.permissions().mode();
        let perms = format!(
            "{}{}{}{}{}{}{}{}{}{} ",
            if self.metadata.is_dir() { 'd' } else { '-' },
            if mode & 0o400 != 0 { 'r' } else { '-' }, // User read
            if mode & 0o200 != 0 { 'w' } else { '-' }, // User write
            if mode & 0o100 != 0 { 'x' } else { '-' }, // User execute
            if mode & 0o040 != 0 { 'r' } else { '-' }, // Group read
            if mode & 0o020 != 0 { 'w' } else { '-' }, // Group write
            if mode & 0o010 != 0 { 'x' } else { '-' }, // Group execute
            if mode & 0o004 != 0 { 'r' } else { '-' }, // Others read
            if mode & 0o002 != 0 { 'w' } else { '-' }, // Others write
            if mode & 0o001 != 0 { 'x' } else { '-' }, // Others execute
        );

        queue!(
            buffer,
            if highlight {
                SetBackgroundColor(Color::DarkGrey)
            } else {
                SetBackgroundColor(Color::Reset)
            },
            SetForegroundColor(Color::Blue),
            Print(perms),
            SetForegroundColor(Color::Yellow),
            Print(format!(
                "{: <8}",
                if self.metadata.is_dir() {
                    "-".to_string()
                } else {
                    format!("{:.1}{}", self.metadata.size() as f32 / divisor, symbol)
                }
            )),
            if self.metadata.is_dir() {
                SetForegroundColor(Color::Blue)
            } else {
                SetForegroundColor(Color::Reset)
            },
            Print(format!(" {: <32}\r\n", self.name)),
            ResetColor
        )?;

        Ok(())
    }
}
