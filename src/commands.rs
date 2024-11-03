use crossterm::{
    execute,
    style::{Color, SetBackgroundColor},
    terminal::{self, LeaveAlternateScreen},
};
use std::{cell::RefCell, fs, io, path::PathBuf, process::exit, rc::Rc};

use crate::fm::FileManager;
use anyhow::{bail, Result};

pub trait Command {
    fn execute(&self, fm: Rc<RefCell<FileManager>>) -> Result<()>;
}

pub struct QuitCommand;

impl Command for QuitCommand {
    fn execute(&self, _fm: Rc<RefCell<FileManager>>) -> Result<()> {
        execute!(io::stdout(), LeaveAlternateScreen)?;
        terminal::disable_raw_mode()?;
        exit(0);
    }
}

pub struct MoveCursorUpCommand;

impl Command for MoveCursorUpCommand {
    fn execute(&self, fm: Rc<RefCell<FileManager>>) -> Result<()> {
        let mut fm = fm.borrow_mut();
        let entries_len = fm.entries().len();
        let position = fm.position();

        if position == 0 {
            fm.set_position(entries_len - 1);
        } else {
            fm.set_position(position - 1);
        }

        Ok(())
    }
}

pub struct MoveCursorDownCommand;

impl Command for MoveCursorDownCommand {
    fn execute(&self, fm: Rc<RefCell<FileManager>>) -> Result<()> {
        let mut fm = fm.borrow_mut();
        let position = fm.position();
        if position == fm.entries().len() - 1 {
            fm.set_position(0);
        } else {
            fm.set_position(position + 1);
        }

        Ok(())
    }
}

pub struct EnterToDirCommand;

impl Command for EnterToDirCommand {
    fn execute(&self, fm: Rc<RefCell<FileManager>>) -> Result<()> {
        let mut fm = fm.borrow_mut();
        let position = fm.position();

        if let Some(entry) = fm.entries().get(position).cloned() {
            if entry.metadata.is_file() {
                bail!("Entry is a file instead of dir");
            }

            fm.position_history_mut().push(position);
            fm.current_path_mut().push(&entry.name);

            match fm.update_entries() {
                Ok(_) => {
                    fm.set_position(0);
                }
                Err(_) => {
                    fm.current_path_mut().pop();
                    execute!(fm.buffer_mut(), SetBackgroundColor(Color::Blue)).unwrap();
                }
            }
        }

        Ok(())
    }
}

pub struct GoToParentDirCommand;

impl Command for GoToParentDirCommand {
    fn execute(&self, fm: Rc<RefCell<FileManager>>) -> Result<()> {
        let mut fm = fm.borrow_mut();

        if let Some(parent_dir) = fm.current_path().parent().map(|path| path.to_path_buf()) {
            fm.set_current_path(parent_dir.into());
        }

        let position = fm.position_history_mut().pop().unwrap_or(0);

        fm.set_position(position);

        fm.update_entries()?;

        Ok(())
    }
}

pub struct ShowFileCommand;

impl Command for ShowFileCommand {
    fn execute(&self, fm: Rc<RefCell<FileManager>>) -> Result<()> {
        let fm_borrow = fm.borrow();

        let current_path = fm_borrow.current_path();
        let entry = fm_borrow.current_entry();

        if !entry.metadata.is_file() {
            bail!("Only files are allowed");
        }

        let mut file_path = PathBuf::from(current_path);
        file_path.push(PathBuf::from(&entry.name));
        let content = fs::read_to_string(&file_path)?;
        let content_with_crlf = content.replace("\n", "\r\n");

        drop(fm_borrow);

        let mut fm_mut = fm.borrow_mut();
        fm_mut.write_to_buffer(content_with_crlf)?;
        fm_mut.toggle_file_view();

        Ok(())
    }
}
