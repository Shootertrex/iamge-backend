use std::fs;
use std::io::Error;
use std::path::{Path, PathBuf};

pub fn load_filesystem_elements(directory: &Path, is_file: bool) -> Result<Vec<PathBuf>, Error> {
    let mut files: Vec<PathBuf> = Vec::new();
    let paths = fs::read_dir(directory)?;

    for maybe_dir_entry in paths {
        let path = (maybe_dir_entry)?.path();

        if path.is_dir() == is_file {
            continue;
        }

        files.push(path);
    }

    Ok(files)
}

pub fn delete_file(file: &Path) -> Result<(), Error> {
    fs::remove_file(file)?;
    Ok(())
}

pub fn move_file(from_file: &Path, to_file: &Path) -> Result<(), Error> {
    fs::rename(from_file, to_file)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::filesystem::*;
    use std::fs;
    use std::fs::File;
    use std::io::ErrorKind;
    use std::path::{Path, PathBuf};
    use tempdir::TempDir;

    #[test]
    fn ensure_files_are_loaded() {
        let expected_files = vec![
            PathBuf::from(r"./images/file1.jpg"),
            PathBuf::from(r"./images/file2.jpg"),
        ];

        let actual_files =
            load_filesystem_elements(Path::new("./images"), true).expect("Found empty list!");

        assert_filesystem_elements(&actual_files, &expected_files);
    }

    #[test]
    fn ensure_folders_are_loaded() {
        let expected_folders = vec![
            PathBuf::from(r"./images/testFolder"),
            PathBuf::from(r"./images/testFolder2"),
        ];

        let actual_folders =
            load_filesystem_elements(Path::new("./images"), false).expect("Found empty list!");

        assert_filesystem_elements(&actual_folders, &expected_folders);
    }

    #[test]
    fn ensure_invalid_folders_are_caught() {
        let expected_error = ErrorKind::NotFound;

        let actual_error = load_filesystem_elements(Path::new("./invalid_directory"), false)
            .err()
            .unwrap();

        assert_eq!(actual_error.kind(), expected_error);
    }

    fn assert_filesystem_elements(
        actual_elements: &Vec<PathBuf>,
        expected_elements: &Vec<PathBuf>,
    ) {
        assert_eq!(actual_elements.len(), expected_elements.len());
        for expected in expected_elements {
            assert!(actual_elements.contains(expected));
        }
    }

    #[test]
    fn ensure_delete_file_deletes_it() {
        let dir = TempDir::new("unit_test").unwrap();
        let file1 = "file1.txt";
        let file2 = "file2.txt";
        File::create(dir.path().join(file1)).unwrap();
        File::create(dir.path().join(file2)).unwrap();

        assert!(!delete_file(&dir.path().join(file1)).is_err());

        assert!(fs::read(dir.path().join(file1)).is_err());
        assert!(fs::read(dir.path().join(file2)).is_ok());
    }

    #[test]
    fn ensure_error_thrown_when_no_file_to_delete() {
        let dir = TempDir::new("unit_test").unwrap();
        let file1 = "file1.txt";

        assert!(delete_file(&dir.path().join(file1)).is_err());
    }

    #[test]
    fn ensure_file_is_moved() {
        let from_dir = TempDir::new("unit_test").unwrap();
        let to_dir = TempDir::new("unit_test").unwrap();
        let file1 = "file1.txt";
        let file2 = "file2.txt";
        File::create(from_dir.path().join(file1)).unwrap();
        File::create(from_dir.path().join(file2)).unwrap();

        assert!(!move_file(
            &from_dir.path().join(file1),
            &to_dir.path().join(file1)
        )
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

        assert!(move_file(
            &from_dir.path().join(fake_file),
            &to_dir.path().join(fake_file)
        )
        .is_err());

        assert!(fs::read(from_dir.path().join(file1)).is_ok());
        assert!(fs::read(to_dir.path().join(file1)).is_err());
    }
}
