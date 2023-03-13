use std::fs;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

#[derive(Default, Clone)]
pub struct Filesystem {}

pub trait FilesystemIO {
    fn load_filesystem_elements(
        &self,
        directory: &Path
    ) -> Result<(Vec<PathBuf>, Vec<PathBuf>), Error>;
    fn delete_file(&self, file: &Path) -> Result<(), Error>;
    fn move_file(&self, from_file: &Path, to_file: &Path) -> Result<(), Error>;
    fn add_folder(&self, folder: &str) -> Result<PathBuf, Error>;
}

impl Filesystem {
    pub fn new() -> Filesystem {
        Filesystem {}
    }
}

impl FilesystemIO for Filesystem {
    fn load_filesystem_elements(
        &self,
        directory: &Path
    ) -> Result<(Vec<PathBuf>, Vec<PathBuf>), Error> {
        let mut files: Vec<PathBuf> = Vec::new();
        let mut folders: Vec<PathBuf> = Vec::new();
        let paths = fs::read_dir(directory)?;

        for maybe_dir_entry in paths {
            let path = (maybe_dir_entry)?.path();

            if path.is_dir() {
                folders.push(path);
            } else {
                files.push(path);
            }
        }

        folders.sort();
        files.sort();

        Ok((folders, files))
    }

    fn delete_file(&self, file: &Path) -> Result<(), Error> {
        fs::remove_file(file)?;
        Ok(())
    }

    fn move_file(&self, from_file: &Path, to_file: &Path) -> Result<(), Error> {
        if to_file.exists() {
            return Err(Error::from(ErrorKind::AlreadyExists));
        }
        fs::rename(from_file, to_file)?;
        Ok(())
    }

    fn add_folder(&self, folder: &str) -> Result<PathBuf, Error> {
        let new_folder = PathBuf::from(folder);

        if !new_folder.exists() {
            return Err(Error::from(ErrorKind::NotFound));
        }

        Ok(new_folder)
    }
}

#[cfg(test)]
mod tests {
    use crate::filesystem::{Filesystem, FilesystemIO};
    use std::{fs, io::Error};
    use std::fs::File;
    use std::io::ErrorKind;
    use std::path::{Path, PathBuf};
    use tempdir::TempDir;

    fn assert_filesystem_elements(
        actual_elements: (Vec<PathBuf>, Vec<PathBuf>),
        expected_elements: (Vec<PathBuf>, Vec<PathBuf>),
    ) {
        assert_eq!(actual_elements.0.len(), expected_elements.0.len());
        for expected in expected_elements.0 {
            assert!(actual_elements.0.contains(&expected));
        }
        assert_eq!(actual_elements.1.len(), expected_elements.1.len());
        for expected in expected_elements.1 {
            assert!(actual_elements.1.contains(&expected));
        }
    }

    #[test]
    fn ensure_files_and_files_are_loaded() {
        let expected_files = (vec![
            PathBuf::from(r"./images/testFolder"),
            PathBuf::from(r"./images/testFolder2"),
        ],
        vec![
            PathBuf::from(r"./images/file1.jpg"),
            PathBuf::from(r"./images/file2.jpg"),
        ]);

        let actual_files = Filesystem::new()
            .load_filesystem_elements(Path::new("./images"))
            .expect("Found empty list!");

        assert_filesystem_elements(actual_files, expected_files);
    }

    #[test]
    fn ensure_invalid_folders_are_caught() {
        let expected_error = ErrorKind::NotFound;

        let actual_error = Filesystem::new()
            .load_filesystem_elements(Path::new("./invalid_directory"))
            .err()
            .unwrap();

        assert_eq!(actual_error.kind(), expected_error);
    }

    #[test]
    fn ensure_delete_file_deletes_it() {
        let dir = TempDir::new("unit_test").unwrap();
        let file1 = "file1.txt";
        let file2 = "file2.txt";
        File::create(dir.path().join(file1)).unwrap();
        File::create(dir.path().join(file2)).unwrap();

        assert!(!Filesystem::new()
            .delete_file(&dir.path().join(file1))
            .is_err());

        assert!(fs::read(dir.path().join(file1)).is_err());
        assert!(fs::read(dir.path().join(file2)).is_ok());
    }

    #[test]
    fn ensure_error_thrown_when_no_file_to_delete() {
        let dir = TempDir::new("unit_test").unwrap();
        let file1 = "file1.txt";

        assert!(Filesystem::new()
            .delete_file(&dir.path().join(file1))
            .is_err());
    }

    #[test]
    fn ensure_error_thrown_when_file_already_exists() {
        let from_dir = TempDir::new("unit_test").unwrap();
        let to_dir = TempDir::new("unit_test").unwrap();
        let file1 = "file1.txt";
        let file2 = "file1.txt";
        File::create(from_dir.path().join(file1)).unwrap();
        File::create(to_dir.path().join(file2)).unwrap();

        let actual = Filesystem::new()
            .move_file(&from_dir.path().join(file1), &to_dir.path().join(file1)).unwrap_err();
        let expected_error = Error::from(ErrorKind::AlreadyExists);

        assert_eq!(expected_error.kind(), actual.kind());
    }

    #[test]
    fn ensure_file_is_moved() {
        let from_dir = TempDir::new("unit_test").unwrap();
        let to_dir = TempDir::new("unit_test").unwrap();
        let file1 = "file1.txt";
        let file2 = "file2.txt";
        File::create(from_dir.path().join(file1)).unwrap();
        File::create(from_dir.path().join(file2)).unwrap();

        assert!(!Filesystem::new()
            .move_file(&from_dir.path().join(file1), &to_dir.path().join(file1))
            .is_err());

        assert!(fs::read(from_dir.path().join(file1)).is_err());
        assert!(fs::read(to_dir.path().join(file1)).is_ok());
        assert!(fs::read(from_dir.path().join(file2)).is_ok());
        assert!(fs::read(to_dir.path().join(file2)).is_err());
    }

    #[test]
    fn ensure_no_file_is_moved_when_file_not_found() {
        let from_dir = TempDir::new("unit_test").unwrap();
        let to_dir = TempDir::new("unit_test").unwrap();
        let file1 = "file1.txt";
        let fake_file = "fake_file.txt";
        File::create(from_dir.path().join(file1)).unwrap();

        assert!(Filesystem::new()
            .move_file(
                &from_dir.path().join(fake_file),
                &to_dir.path().join(fake_file)
            )
            .is_err());

        assert!(fs::read(from_dir.path().join(file1)).is_ok());
        assert!(fs::read(to_dir.path().join(file1)).is_err());
    }

    #[test]
    fn ensure_single_folder_is_added_when_add_folder_is_called() {
        let expected_folders = (vec![PathBuf::from(r"./images/testFolder")], vec![]);

        let actual_folders = (
            vec![Filesystem::new()
                .add_folder(&"./images/testFolder".to_owned())
                .expect("Found empty list!")],
            vec![]
        );

        assert_filesystem_elements(actual_folders, expected_folders);
    }

    #[test]
    fn ensure_not_found_error_thrown_when_add_folder_is_called_with_bad_path() {
        let expected_error = ErrorKind::NotFound;

        let actual_error = Filesystem::new()
            .add_folder(&"./images/bad_folder".to_owned())
            .err()
            .unwrap();

        assert_eq!(actual_error.kind(), expected_error);
    }
}
