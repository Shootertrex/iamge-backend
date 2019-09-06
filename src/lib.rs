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

    pub fn load_folders_and_files(&mut self, directory: String) {
        self.files = match filesystem::load_filesystem_elements(Path::new(&directory), true) {
            Some(valid_files) => valid_files,
            None => /* exit for now */return,
        };

        self.folders = match filesystem::load_filesystem_elements(Path::new(&directory), true) {
            Some(valid_files) => valid_files,
            None => /* exit for now */return,
        };

        self.pwd = directory;
    }

    pub fn load_external_folders(&mut self, directory: String) {
        self.folders = match filesystem::load_filesystem_elements(Path::new(&directory), true) {
            Some(valid_files) => valid_files,
            None => /* exit for now */return,
        };
    }

    pub fn add_folder(&mut self, directory: String) {
        let new_folder = PathBuf::from(&directory);

        if !new_folder.exists() {
            // some error code
            return;
        }

        self.folders.push(new_folder);
    }
    
    pub fn clear_folders(&mut self) {
        self.folders = Vec::new();
    }

}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
