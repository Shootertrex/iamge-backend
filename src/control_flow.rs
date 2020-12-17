use crate::filesystem::{Filesystem, FilesystemIO};
use std::io::Error;
use std::path::{Path, PathBuf};

pub struct Move {
    pub current_file_location: PathBuf,
    pub previous_file_location: PathBuf,
    pub filesystem_helper: Box<dyn FilesystemIO>,
}

impl Move {
    pub fn new(current_location: PathBuf, previous_location: PathBuf) -> Move {
        Move {
            current_file_location: current_location,
            previous_file_location: previous_location,
            filesystem_helper: Box::new(Filesystem::new()),
        }
    }
}

impl Controllable for Move {
    fn undo(&self) -> Result<(), Error> {
        self.filesystem_helper
            .move_file(&self.current_file_location, &self.previous_file_location)?;

        Ok(())
    }

    fn redo(&self) -> Result<(), Error> {
        self.filesystem_helper.move_file(
            Path::new(&self.previous_file_location),
            Path::new(&self.current_file_location),
        )?;

        Ok(())
    }
}

pub struct Skip {
    // does nothing
}

impl Skip {
    pub fn new() -> Skip {
        Skip {}
    }
}

impl Controllable for Skip {
    fn undo(&self) -> Result<(), Error> {
        // do nothing except decrement pointer
        Ok(())
    }

    fn redo(&self) -> Result<(), Error> {
        // do nothing except increment pointer
        Ok(())
    }
}

pub trait Controllable {
    fn undo(&self) -> Result<(), Error>;
    fn redo(&self) -> Result<(), Error>;
}
