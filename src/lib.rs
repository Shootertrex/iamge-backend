use crate::control_flow::{Move, Skip};
use crate::filesystem::{Filesystem, FilesystemIO};
use control_flow::Controllable;
use std::io::Error;
use std::path::{Path, PathBuf};

mod control_flow;
mod filesystem;

pub struct Backend {
    files: Vec<PathBuf>,
    folders: Vec<PathBuf>,
    pwd: String,
    control_flow: Vec<Box<dyn Controllable>>,
    filesystem_helper: Box<dyn FilesystemIO>,
}

impl Default for Backend {
    fn default() -> Self {
        Backend::new()
    }
}

impl Backend {
    pub fn new() -> Backend {
        Backend {
            files: Vec::new(),
            folders: Vec::new(),
            pwd: String::new(),
            control_flow: Vec::new(),
            filesystem_helper: Box::new(Filesystem::new()),
        }
    }

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
        self.files = self
            .filesystem_helper
            .load_filesystem_elements(Path::new(&directory), true)?;
        self.folders = self
            .filesystem_helper
            .load_filesystem_elements(Path::new(&directory), false)?;
        self.pwd = directory;

        Ok(())
    }

    pub fn load_external_folders(&mut self, directory: String) -> Result<(), Error> {
        self.folders = self
            .filesystem_helper
            .load_filesystem_elements(Path::new(&directory), false)?;

        Ok(())
    }

    pub fn move_file(&mut self, from_file: String, to_file: String) -> Result<(), Error> {
        self.filesystem_helper
            .move_file(Path::new(&from_file), Path::new(&to_file))?;

        self.control_flow
            .push(Box::new(Move::new(from_file, to_file)));

        Ok(())
    }

    pub fn add_folder(&mut self, directory: String) -> Result<(), Error> {
        let new_folder = self.filesystem_helper.add_folder(&directory)?;
        self.folders.push(new_folder);

        Ok(())
    }

    pub fn clear_folders(&mut self) {
        self.folders = Vec::new();
    }

    pub fn delete_file(&mut self, file_path: String) -> Result<(), Error> {
        self.filesystem_helper.delete_file(Path::new(&file_path))
    }

    pub fn skip(&mut self) {
        // move pointer forward
        self.control_flow.push(Box::new(Skip::new()));
    }
}

#[cfg(test)]
mod tests {
    use crate::filesystem::FilesystemIO;
    use crate::Backend;
    use std::io::{Error, ErrorKind};
    use std::path::{Path, PathBuf};

    struct FilesystemMock {
        folders: Vec<PathBuf>,
        files: Vec<PathBuf>,
    }

    impl FilesystemMock {
        fn new() -> FilesystemMock {
            FilesystemMock {
                folders: Vec::new(),
                files: Vec::new(),
            }
        }
    }

    impl FilesystemIO for FilesystemMock {
        fn load_filesystem_elements(
            &self,
            directory: &Path,
            is_file: bool,
        ) -> Result<Vec<PathBuf>, Error> {
            if is_file {
                Ok(self.files.clone())
            } else if !is_file {
                Ok(self.folders.clone())
            } else {
                Err(Error::from(ErrorKind::NotFound))
            }
        }
        fn delete_file(&self, file: &Path) -> Result<(), Error> {
            Ok(())
        }
        fn move_file(&self, from_file: &Path, to_file: &Path) -> Result<(), Error> {
            Ok(())
        }
        fn add_folder(&self, folder: &str) -> Result<PathBuf, Error> {
            if self.folders.len() == 1 {
                Ok(self.folders[0].clone())
            } else {
                Err(Error::from(ErrorKind::NotFound))
            }
        }
    }

    #[test]
    fn ensure_files_and_folders_are_populated_when_loading_all() {
        let mut test_backend = Backend::new();
        let mut filesystem_mock = FilesystemMock::new();
        let expected_folders = build_folders();
        let expected_files = build_files();
        filesystem_mock.folders = expected_folders.clone();
        filesystem_mock.files = expected_files.clone();
        test_backend.filesystem_helper = Box::new(filesystem_mock);

        test_backend
            .load_folders_and_files("./testFolder".to_owned())
            .unwrap();

        let actual_folders = test_backend.folders();
        let actual_files = test_backend.files();
        assert_vectors(&actual_folders, &expected_folders);
        assert_vectors(&actual_files, &expected_files);
    }

    #[test]
    fn ensure_folders_are_populated_when_loading_external_folders() {
        let mut test_backend = Backend::new();
        let mut filesystem_mock = FilesystemMock::new();
        let expected_folders = build_folders();
        filesystem_mock.folders = expected_folders.clone();
        test_backend.filesystem_helper = Box::new(filesystem_mock);

        test_backend
            .load_external_folders("./testFolder".to_owned())
            .unwrap();

        let actual_folders = test_backend.folders();
        let actual_files = test_backend.files();
        assert_vectors(&actual_folders, &expected_folders);
    }

    #[test]
    fn ensure_file_is_moved_when_move_file_is_called() {
        let mut test_backend = Backend::new();
        let mut filesystem_mock = FilesystemMock::new();
        test_backend.filesystem_helper = Box::new(filesystem_mock);

        test_backend
            .move_file("./fromFolder".to_owned(), "./toFolder".to_owned())
            .unwrap();

        //fill in later
    }

    #[test]
    fn ensure_file_is_added_when_adding_a_folder() {
        let mut test_backend = Backend::new();
        let mut filesystem_mock = FilesystemMock::new();
        let expected_folders = vec![PathBuf::from("./folder1")];
        filesystem_mock.folders = expected_folders.clone();
        test_backend.filesystem_helper = Box::new(filesystem_mock);

        test_backend.add_folder("./testFolder".to_owned()).unwrap();

        let actual_folders = test_backend.folders();
        assert_vectors(&actual_folders, &expected_folders);
    }

    #[test]
    fn ensure_folders_are_cleared_when_clearing_folders() {
        let original_folders = build_folders();
        let expected_files = build_files();
        let mut test_backend = Backend::new();
        test_backend.folders = original_folders.clone();
        test_backend.files = expected_files.clone();

        test_backend.clear_folders();

        let actual_folders = test_backend.folders();
        let actual_files = test_backend.files();
        assert_vectors(&actual_folders, &Vec::new());
        assert_vectors(&actual_files, &expected_files);
    }

    #[test]
    fn ensure_file_is_removed_when_deleting_file() {
        let expected_folders = build_folders();
        let expected_files = build_files();
        let mut test_backend = Backend::new();
        test_backend.filesystem_helper = Box::new(FilesystemMock::new());
        test_backend.folders = expected_folders.clone();
        test_backend.files = expected_files.clone();

        test_backend.delete_file("./file1.png".to_owned());

        let actual_folders = test_backend.folders();
        let actual_files = test_backend.files();
        assert_vectors(&actual_folders, &expected_folders);
        assert_vectors(&actual_files, &expected_files);
    }

    fn assert_vectors(actual_vector: &Vec<PathBuf>, expected_vector: &Vec<PathBuf>) {
        assert_eq!(actual_vector.len(), expected_vector.len());
        for expected in expected_vector {
            assert!(actual_vector.contains(expected));
        }
    }

    fn build_folders() -> Vec<PathBuf> {
        vec![
            PathBuf::from("./folder1"),
            PathBuf::from("./folder2"),
            PathBuf::from("./folder3"),
        ]
    }

    fn build_files() -> Vec<PathBuf> {
        vec![
            PathBuf::from("./file1.png"),
            PathBuf::from("./file2.png"),
            PathBuf::from("./file3.png"),
        ]
    }
}
