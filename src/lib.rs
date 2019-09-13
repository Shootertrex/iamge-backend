use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
mod filesystem;

pub struct Backend {
    files: Vec<PathBuf>,
    folders: Vec<PathBuf>,
    pwd: String,
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

    pub fn add_folder(&mut self, directory: String) -> Result<(), Error> {
        let new_folder = PathBuf::from(&directory);

        if !new_folder.exists() {
            return Err(Error::from(ErrorKind::NotFound));
        }

        self.folders.push(new_folder);

        Ok(())
    }

    pub fn clear_folders(&mut self) {
        self.folders = Vec::new();
    }

    pub fn delete_file(&mut self, file_path: String) -> Result<(), Error> {
        filesystem::delete_file(Path::new(&file_path))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
