use crate::control_flow::{Move, Skip};
use crate::filesystem::{Filesystem, FilesystemIO};
use control_flow::Controllable;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

mod control_flow;
mod filesystem;

pub struct Backend {
    pub files: Vec<PathBuf>,
    pub folders: Vec<PathBuf>,
    pub pwd: String,
    pub current_file_index: usize,
    undo_stack: Vec<Box<dyn Controllable>>,
    redo_stack: Vec<Box<dyn Controllable>>,
    pub filesystem_helper: Box<dyn FilesystemIO>,
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
            current_file_index: 0,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            filesystem_helper: Box::new(Filesystem::new()),
        }
    }

    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    pub fn get_current_file(&self) -> &PathBuf {
        &self.files[self.current_file_index]
    }

    pub fn load_folders_and_files(&mut self, directory: String) -> Result<(), Error> {
        let clean_directory = directory.trim();

        self.files = self
            .filesystem_helper
            .load_filesystem_elements(Path::new(&clean_directory), true)?;
        self.folders = self
            .filesystem_helper
            .load_filesystem_elements(Path::new(&clean_directory), false)?;
        self.pwd = directory;

        Ok(())
    }

    pub fn load_external_folders(&mut self, directory: String) -> Result<(), Error> {
        self.folders = self
            .filesystem_helper
            .load_filesystem_elements(Path::new(&directory.trim()), false)?;

        Ok(())
    }

    pub fn add_folder(&mut self, directory: String) -> Result<(), Error> {
        let new_folder = self.filesystem_helper.add_folder(&directory.trim())?;
        self.folders.push(new_folder);

        Ok(())
    }

    pub fn clear_folders(&mut self) {
        self.folders = Vec::new();
    }

    pub fn delete_file(&mut self) -> Result<(), Error> {
        match self.filesystem_helper.delete_file(self.get_current_file()) {
            Ok(_) => {
                self.undo_stack.push(Box::new(Skip::new()));

                Ok(())
            }
            Err(error) => Err(error),
        }
    }

    pub fn move_file(&mut self, to_folder: PathBuf) -> Result<(), Error> {
        if self.files.is_empty() {
            return Err(Error::from(ErrorKind::NotFound));
        }

        let from_file = self.get_current_file().clone();
        let destination = Self::build_destination(to_folder, &from_file)?;

        self.filesystem_helper.move_file(&from_file, &destination)?;

        self.undo_stack
            .push(Box::new(Move::new(from_file, destination)));

        Ok(())
    }

    fn build_destination(mut to_folder: PathBuf, from_file: &PathBuf) -> Result<PathBuf, Error> {
        let file_name = match from_file.file_name() {
            Some(some_file) => some_file,
            None => {
                return Err(Error::from(ErrorKind::InvalidData));
            }
        };
        to_folder.push(file_name);

        Ok(to_folder)
    }

    pub fn increment(&mut self) -> Result<(), String> {
        Self::is_end_of_files(self.current_file_index, self.file_count())?;
        self.current_file_index += 1;
        self.undo_stack.push(Box::new(Skip::new()));

        Ok(())
    }

    fn is_end_of_files(file_index: usize, file_count: usize) -> Result<(), String> {
        if file_index + 1 >= file_count {
            return Err("Reached end of files!".to_owned());
        }
        Ok(())
    }

    pub fn undo(&mut self) -> Result<(), Error> {
        match self.undo_stack.pop() {
            Some(item) => {
                let result = item.undo();
                self.redo_stack.push(item);
                self.current_file_index -= 1;
                result
            }
            None => Ok(()),
        }
    }

    pub fn redo(&mut self) -> Result<(), Error> {
        match self.redo_stack.pop() {
            Some(item) => {
                let result = item.redo();
                self.undo_stack.push(item);
                self.current_file_index += 1;
                result
            }
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::control_flow::Move;
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
            _directory: &Path,
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
        fn delete_file(&self, _file: &Path) -> Result<(), Error> {
            Ok(())
        }
        fn move_file(&self, _from_file: &Path, _to_file: &Path) -> Result<(), Error> {
            Ok(())
        }
        fn add_folder(&self, _folder: &str) -> Result<PathBuf, Error> {
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

        let actual_folders = test_backend.folders;
        let actual_files = test_backend.files;
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

        let actual_folders = test_backend.folders;
        assert_vectors(&actual_folders, &expected_folders);
    }

    #[test]
    fn ensure_file_is_moved_when_move_file_is_called() {
        let filesystem_mock = FilesystemMock::new();
        let expected_files = build_files();
        let mut test_backend = Backend::new();
        test_backend.filesystem_helper = Box::new(filesystem_mock);
        test_backend.files = expected_files.clone();
        assert_eq!(test_backend.undo_stack.len(), 0);

        test_backend
            .move_file(PathBuf::from("./toFolder"))
            .unwrap();

        assert_eq!(test_backend.undo_stack.len(), 1);
    }

    #[test]
    fn ensure_error_is_thrown_when_no_files_loaded_when_moving() {
        let mut test_backend = Backend::new();

        assert!(test_backend
            .move_file(PathBuf::from("./toFolder"))
            .is_err());
    }

    #[test]
    fn ensure_file_is_added_when_adding_a_folder() {
        let mut test_backend = Backend::new();
        let mut filesystem_mock = FilesystemMock::new();
        let expected_folders = vec![PathBuf::from("./folder1")];
        filesystem_mock.folders = expected_folders.clone();
        test_backend.filesystem_helper = Box::new(filesystem_mock);

        test_backend.add_folder("./testFolder".to_owned()).unwrap();

        let actual_folders = test_backend.folders;
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

        let actual_folders = test_backend.folders;
        let actual_files = test_backend.files;
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
        assert_eq!(test_backend.undo_stack.len(), 0);

        test_backend.delete_file().expect("delete failed!");

        let actual_folders = &test_backend.folders;
        let actual_files = &test_backend.files;
        assert_vectors(&actual_folders, &expected_folders);
        assert_vectors(&actual_files, &expected_files);
        assert_eq!(test_backend.undo_stack.len(), 1);
    }

    #[test]
    fn ensure_pointer_is_moved_forward_when_file_is_skipped() {
        let expected_files = build_files();
        let mut test_backend = Backend::new();
        test_backend.files = expected_files.clone();
        let expected_index = 1;
        assert_eq!(
            test_backend.get_current_file(),
            &expected_files[expected_index - 1]
        );
        assert_eq!(test_backend.undo_stack.len(), 0);

        test_backend.increment().expect("Skipping failed");

        assert_eq!(test_backend.current_file_index, expected_index);
        assert_eq!(
            test_backend.get_current_file(),
            &expected_files[expected_index]
        );
        assert_eq!(test_backend.undo_stack.len(), 1);
    }

    #[test]
    fn ensure_error_is_thrown_when_index_out_of_bounds_when_skipping() {
        let expected_files = build_files();
        let mut test_backend = Backend::new();
        test_backend.files = expected_files.clone();
        test_backend.current_file_index = 2;
        let expected_index = 3;
        assert_eq!(
            test_backend.get_current_file(),
            &expected_files[expected_index - 1]
        );

        assert!(test_backend.increment().is_err());
    }

    #[test]
    fn ensure_undo_stack_is_popped_and_redo_stack_is_pushed_when_undoing() {
        let mut test_backend = Backend::new();
        let filesystem_mock = FilesystemMock::new();
        let mut undo_element = Move::new(PathBuf::from("a"), PathBuf::from("b"));
        undo_element.filesystem_helper = Box::new(filesystem_mock);
        test_backend.undo_stack.push(Box::new(undo_element));
        test_backend.current_file_index = 2;

        test_backend.undo().expect("undo failed");

        assert_eq!(test_backend.redo_stack.len(), 1);
        assert_eq!(test_backend.undo_stack.len(), 0);
        assert_eq!(test_backend.current_file_index, 1);
    }

    #[test]
    fn ensure_nothing_happens_when_undo_stack_is_empty() {
        let mut test_backend = Backend::new();
        test_backend.current_file_index = 0;

        test_backend.undo().expect("undo failed");

        assert_eq!(test_backend.redo_stack.len(), 0);
        assert_eq!(test_backend.undo_stack.len(), 0);
        assert_eq!(test_backend.current_file_index, 0);
    }

    #[test]
    fn ensure_redo_stack_is_popped_and_undo_stack_is_pushed_when_redoing() {
        let mut test_backend = Backend::new();
        let filesystem_mock = FilesystemMock::new();
        let mut redo_element = Move::new(PathBuf::from("a"), PathBuf::from("b"));
        redo_element.filesystem_helper = Box::new(filesystem_mock);
        test_backend.redo_stack.push(Box::new(redo_element));
        test_backend.current_file_index = 0;

        test_backend.redo().expect("redo failed");

        assert_eq!(test_backend.redo_stack.len(), 0);
        assert_eq!(test_backend.undo_stack.len(), 1);
        assert_eq!(test_backend.current_file_index, 1);
    }

    #[test]
    fn ensure_nothing_happens_when_redo_stack_is_empty() {
        let mut test_backend = Backend::new();
        test_backend.current_file_index = 0;

        test_backend.redo().expect("redo failed");

        assert_eq!(test_backend.redo_stack.len(), 0);
        assert_eq!(test_backend.undo_stack.len(), 0);
        assert_eq!(test_backend.current_file_index, 0);
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
