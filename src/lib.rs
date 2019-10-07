use crate::control_flow::{Move, Skip};
use control_flow::Controllable;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

mod control_flow;
mod filesystem;

pub struct Backend {
    files: Vec<PathBuf>,
    folders: Vec<PathBuf>,
    pwd: String,
    control_flow: Vec<Box<dyn Controllable>>,
}

impl Backend {
    pub fn files(&self) -> &Vec<PathBuf> {
        &self.files
    }

    pub fn folders(&self) -> &Vec<PathBuf> {
        &self.folders
    }

    pub fn pwd(&self) -> &String {
        &self.pwd
    }

    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    pub fn load_folders_and_files(&mut self, directory: String) -> Result<(), Error> {
        self.files = filesystem::load_filesystem_elements(Path::new(&directory), true)?;
        self.folders = filesystem::load_filesystem_elements(Path::new(&directory), true)?;
        self.pwd = directory;

        Ok(())
    }

    pub fn load_external_folders(&mut self, directory: String) -> Result<(), Error> {
        self.folders = filesystem::load_filesystem_elements(Path::new(&directory), true)?;

        Ok(())
    }

    pub fn move_file(&mut self, from_file: String, to_file: String) -> Result<(), Error> {
        filesystem::move_file(Path::new(&from_file), Path::new(&to_file))?;

        self.control_flow
            .push(Box::new(Move::new(from_file, to_file)));

        Ok(())
    }

    pub fn add_folder(&mut self, directory: String) -> Result<(), Error> {
        let new_folder = filesystem::add_folder(&directory)?;
        self.folders.push(new_folder);

        Ok(())
    }

    pub fn clear_folders(&mut self) {
        self.folders = Vec::new();
    }

    pub fn delete_file(&mut self, file_path: String) -> Result<(), Error> {
        filesystem::delete_file(Path::new(&file_path))
    }

    pub fn skip(&mut self) {
        // move pointer forward
        self.control_flow.push(Box::new(Skip::new()));
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
