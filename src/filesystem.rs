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

#[cfg(test)]
mod tests {
    use crate::filesystem::load_filesystem_elements;
    use std::io::ErrorKind;
    use std::path::{Path, PathBuf};

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
}
