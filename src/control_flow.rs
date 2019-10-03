use crate::filesystem;
use std::io::Error;
use std::path::Path;

//how to undo a delete in rust? not possible?
pub struct Delete {
    pub current_file_location: i32,
    pub previous: i32,
}

pub struct Move {
    pub current_file_location: String,
    pub previous_file_location: String,
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

trait Controllable {
    fn undo(&self) -> Result<(), Error>;
    fn redo(&self) -> Result<(), Error>;
}
