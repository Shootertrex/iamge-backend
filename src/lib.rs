//! Provides a backend for different operations to make sorting files into various folders
//! easier. It is expected to be used with some kind of frontend.
//!
//! [Backend] allows for several different actions for loading files/folders and moving loaded
//! files into those folders. These actions include:
//! - loading all folders and files from a single directory
//! - loading just folders from a directory
//! - adding a single folder by its path
//! - moving a file
//! - deleting a file
//! - skipping a file
//!
//! All operations[^note] that deal with files can be undone and redone. When these
//! actions are performed, their respective action is added to an undo stack or a redo stack in
//! case the user wishes to playback previous actions.
//!
//! [^note]: Deletions are currently not capable of being undone.

use crate::control_flow::{Controllable, Move, Skip};
use crate::filesystem::{Filesystem, FilesystemIO};
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

mod control_flow;
mod filesystem;

pub struct Backend {
    /// Collection of all files loaded to be sorted.
    pub files: Vec<PathBuf>,
    /// Collection of all folders loaded that files can be sorted into.
    pub folders: Vec<PathBuf>,
    /// The current working directory.
    pub pwd: String,
    /// The index to the current file in the [files vector](Backend::files).
    pub current_file_index: usize,
    undo_stack: Vec<Box<dyn Controllable>>,
    redo_stack: Vec<Box<dyn Controllable>>,
    #[doc(hidden)]
    pub filesystem_helper: Box<dyn FilesystemIO>,
    end_of_files: bool,
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
            end_of_files: false,
        }
    }

    /// Returns the number of files to be sorted.
    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    /// Returns a &[PathBuf] to the current file.
    // TODO: should this return an Option instead of using a default path?
    pub fn get_current_file(&self) -> Option<&PathBuf> {
        if self.current_file_index >= self.file_count() {
            return None;
        }

        Some(&self.files[self.current_file_index])
    }

    /// Loads all files and directories in the specified path.
    ///
    /// Files and folders are loaded into their own vectors and kept in the object's state.
    /// Any files and folders that were previously loaded are cleared and replaced with these new
    /// ones. All other state is cleared as well.
    ///
    /// # Errors
    ///
    /// If there are any I/O errors reading from the specified directory, an error variant will be
    /// returned.
    pub fn load_folders_and_files(&mut self, directory: String) -> Result<(), Error> {
        let clean_directory = directory.trim();

        (self.folders, self.files) = self
            .filesystem_helper
            .load_filesystem_elements(Path::new(&clean_directory))?;
        self.pwd = directory;
        self.current_file_index = 0;
        self.undo_stack = Vec::new();
        self.redo_stack = Vec::new();

        Ok(())
    }

    /// Loads directories in the specified path.
    ///
    /// Directories from this path are set as the available folders to sort into. Files are not
    /// loaded nor cleared. This may be useful in cases where files from one directory should be
    /// sorted elsewhere on the filesystem and not necessarily into sibling folders.
    ///
    /// # Errors
    ///
    /// If there are any I/O errors reading from the specified directory, an error variant will be
    /// returned.
    pub fn load_external_folders(&mut self, directory: String) -> Result<(), Error> {
        // TODO: add function to just get folders
        self.folders = self
            .filesystem_helper
            .load_filesystem_elements(Path::new(&directory.trim()))?
            .0;

        Ok(())
    }

    /// Adds a specified folder to the list of folders where files can be sorted into.
    ///
    /// # Errors
    ///
    /// If there are any I/O errors reading the folder path, an error variant will be
    /// returned.
    pub fn add_folder(&mut self, directory: String) -> Result<(), Error> {
        let new_folder = self.filesystem_helper.add_folder(directory.trim())?;
        self.folders.push(new_folder);

        Ok(())
    }

    /// Clears the currently loaded folders.
    pub fn clear_folders(&mut self) {
        self.folders = Vec::new();
    }

    /// Deletes the current file.
    ///
    /// # Errors
    ///
    /// If there are any I/O errors deleting from the specified file, an error variant will be
    /// returned.
    // TODO: shouldn't this increment like move/skip?
    pub fn delete_file(&mut self) -> Result<(), Error> {
        if let Some(file) = self.get_current_file() {
            match self.filesystem_helper.delete_file(file) {
                Ok(_) => {
                    self.undo_stack.push(Box::new(Skip::new()));
                }
                Err(error) => { return Err(error) },
            }
        }

        Ok(())
    }

    /// Moves the current file to a specified path.
    ///
    /// A `control_flow` action that moves the current file to the specified path. It should be
    /// noted that this method takes a [PathBuf] instead of a [String] like the loading methods.
    /// This is because it is expected that the `to_folder` comes from the selected folder's path,
    /// which is already a [PathBuf].
    ///
    /// # Errors
    ///
    /// If there are any I/O errors moving the file, an error variant will be
    /// returned.
    pub fn move_file(&mut self, to_folder: PathBuf) -> Result<(), Error> {
        if self.files.is_empty() {
            return Err(Error::from(ErrorKind::NotFound));
        }

        if let Some(from_file) = self.get_current_file() {
            let destination = Self::build_destination(to_folder, from_file)?;

            self.filesystem_helper.move_file(from_file, &destination)?;

            println!("incrementing {}", self.current_file_index);
            self.undo_stack
                .push(Box::new(Move::new(from_file.clone(), destination)));
            self.increment()?;
        }

        Ok(())
    }

    fn build_destination(mut to_folder: PathBuf, from_file: &Path) -> Result<PathBuf, Error> {
        let file_name = match from_file.file_name() {
            Some(some_file) => some_file,
            None => {
                return Err(Error::from(ErrorKind::InvalidData));
            }
        };
        to_folder.push(file_name);

        Ok(to_folder)
    }

    fn increment(&mut self) -> Result<(), Error> {
        self.is_end_of_files(self.current_file_index, self.file_count())?;
        self.current_file_index += 1;

        Ok(())
    }

    /// Skips the current file.
    ///
    /// A `control_flow` action that increments the index that points to the current file forward.
    pub fn skip(&mut self) -> Result<(), Error> {
        self.increment()?;
        self.undo_stack.push(Box::new(Skip::new()));

        Ok(())
    }

    fn is_end_of_files(&mut self, file_index: usize, file_count: usize) -> Result<(), Error> {
        match file_index + 1 >= file_count {
            true => {
                self.end_of_files = true;
                Err(Error::from(ErrorKind::UnexpectedEof))
            }
            false => Ok(()),
        }
    }

    /// Undoes the previous action.
    ///
    /// Undoes the previous `control_flow` action and pushes a redo action onto the `redo_stack`.
    ///
    /// # Errors
    ///
    /// If there are any I/O errors while undoing, an error variant will be
    /// returned.
    pub fn undo(&mut self) -> Result<(), Error> {
        match self.undo_stack.pop() {
            Some(item) => {
                let result = item.undo();
                self.redo_stack.push(item);
                if self.end_of_files {
                    self.end_of_files = false;
                } else {
                    self.current_file_index -= 1;
                }

                result
            }
            None => Ok(()),
        }
    }

    /// Redoes the action most recently undone.
    ///
    /// Redoes the last `control_flow` action on the `redo_stack` and pushes an undo action
    /// onto the `undo_stack`. The `redo_stack` gets cleared when any action that isn't a
    /// redo or an undo is performed.
    ///
    /// # Errors
    ///
    /// If there are any I/O errors redoing, an error variant will be
    /// returned.
    pub fn redo(&mut self) -> Result<(), Error> {
        match self.redo_stack.pop() {
            Some(item) => {
                let result = item.redo();
                self.undo_stack.push(item);
                self.increment()?;
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
        ) -> Result<(Vec<PathBuf>, Vec<PathBuf>), Error> {
            Ok((self.folders.clone(), self.files.clone()))
        }
        fn delete_file(&self, _file: &Path) -> Result<(), Error> {
            Ok(())
        }
        fn move_file(&self, _from_file: &Path, _to_file: &Path) -> Result<(), Error> {
            Ok(())
        }
        fn add_folder(&self, _folder: &str) -> Result<PathBuf, Error> {
            match self.folders.len() == 1 {
                true => Ok(self.folders[0].clone()),
                false => Err(Error::from(ErrorKind::NotFound)),
            }
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

        test_backend.move_file(PathBuf::from("./toFolder")).unwrap();

        assert_eq!(test_backend.undo_stack.len(), 1);
        assert_eq!(test_backend.current_file_index, 1);
    }

    #[test]
    fn ensure_error_is_thrown_when_no_files_loaded_when_moving() {
        let mut test_backend = Backend::new();

        assert!(test_backend.move_file(PathBuf::from("./toFolder")).is_err());
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
            test_backend.get_current_file().unwrap(),
            &expected_files[expected_index - 1]
        );
        assert_eq!(test_backend.undo_stack.len(), 0);

        test_backend.skip().expect("Skipping failed");

        assert_eq!(test_backend.current_file_index, expected_index);
        assert_eq!(
            test_backend.get_current_file().unwrap(),
            &expected_files[expected_index]
        );
        assert_eq!(test_backend.undo_stack.len(), 1);
    }

    #[test]
    fn ensure_pointer_is_moved_forward_and_unchanged_undo_stack_when_incrementing() {
        let expected_files = build_files();
        let mut test_backend = Backend::new();
        test_backend.files = expected_files.clone();
        let expected_index = 1;
        assert_eq!(
            test_backend.get_current_file().unwrap(),
            &expected_files[expected_index - 1]
        );
        assert_eq!(test_backend.undo_stack.len(), 0);

        test_backend.increment().expect("Skipping failed");

        assert_eq!(test_backend.current_file_index, expected_index);
        assert_eq!(
            test_backend.get_current_file().unwrap(),
            &expected_files[expected_index]
        );
        assert_eq!(test_backend.undo_stack.len(), 0);
    }

    #[test]
    fn ensure_error_is_thrown_when_index_out_of_bounds_when_skipping() {
        let expected_files = build_files();
        let mut test_backend = Backend::new();
        test_backend.files = expected_files.clone();
        test_backend.current_file_index = 2;
        let expected_index = 3;
        assert_eq!(
            test_backend.get_current_file().unwrap(),
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
    fn ensure_final_image_is_undone_when_end_of_files_hit_and_undo() {
        let filesystem_mock = FilesystemMock::new();
        let expected_files = build_files();
        let mut test_backend = Backend::new();
        test_backend.filesystem_helper = Box::new(filesystem_mock);
        test_backend.files = expected_files.clone();
        assert_eq!(test_backend.undo_stack.len(), 0);
        dbg!(&test_backend.files);
        test_backend.move_file(PathBuf::from("./toFolder")).unwrap();
        test_backend.move_file(PathBuf::from("./toFolder")).unwrap();
        let _ = test_backend.move_file(PathBuf::from("./toFolder")); // ignore result since EoF

        let _ = test_backend.undo(); // ignore result since it tries on real filesystem

        assert_eq!(test_backend.current_file_index, 2);
    }

    #[test]
    fn ensure_redo_stack_is_popped_and_undo_stack_is_pushed_when_redoing() {
        let filesystem_mock = FilesystemMock::new();
        let expected_files = build_files();
        let mut redo_element = Move::new(PathBuf::from("a"), PathBuf::from("b"));
        redo_element.filesystem_helper = Box::new(filesystem_mock);
        let mut test_backend = Backend::new();
        test_backend.redo_stack.push(Box::new(redo_element));
        test_backend.current_file_index = 0;
        test_backend.files = expected_files;

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

    #[test]
    fn ensure_none_is_returned_when_current_file_index_is_out_of_bounds() {
        let mut test_backend = Backend::new();
        test_backend.current_file_index = 10;

        assert_eq!(test_backend.get_current_file().is_none(), true);
    }

    fn assert_vectors(actual_vector: &Vec<PathBuf>, expected_vector: &Vec<PathBuf>) {
        assert_eq!(actual_vector.len(), expected_vector.len());
        for expected in expected_vector {
            assert!(actual_vector.contains(expected));
        }
    }
}

