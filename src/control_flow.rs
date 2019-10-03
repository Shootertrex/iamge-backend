use crate::filesystem;
use std::io::Error;
use std::path::Path;

pub struct Move {
    pub current_file_location: String,
    pub previous_file_location: String,
}

impl Move {
    pub fn new(current_location: String, previous_location: String) -> Move {
        Move {
            current_file_location: current_location,
            previous_file_location: previous_location,
        }
    }
}

impl Controllable for Move {
    fn undo(&self) -> Result<(), Error> {
        filesystem::move_file(
            Path::new(&self.current_file_location),
            Path::new(&self.previous_file_location),
        )?;

        Ok(())
    }

    fn redo(&self) -> Result<(), Error> {
        filesystem::move_file(
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
