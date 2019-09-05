use std::path::{Path};
use std::fs;

pub fn load_filesystem_elements(directory: &Path, is_file: bool) -> Option<Vec<String>> {
    let mut files: Vec<String> = Vec::new();

    let paths = match fs::read_dir(directory){
        Ok(valid_paths) => valid_paths,
        Err(_) => return None,
    };

    for maybe_dir_entry in paths {
        let path = match maybe_dir_entry {
            Ok(valid_dir) => valid_dir.path(),
            Err(_) => continue,
        };

        if path.is_dir() == is_file {
            continue;
        }

        files.push(path
            .into_os_string()
            .to_string_lossy()
            .to_string());
    }

    Some(files)
}

#[cfg(test)]
mod tests {
    use crate::filesystem::load_filesystem_elements;
    use std::path::Path;

    #[test]
    fn ensure_files_are_loaded() {
        let expected_files = vec!["./images/file1.jpg", "./images/file2.jpg"];

        let actual_files = load_filesystem_elements(Path::new("./images"), true).expect("Found empty list!");

        assert_filesystem_elements(&actual_files, &expected_files);
    }

    #[test]
    fn ensure_folders_are_loaded() {
        let expected_folders = vec!["./images/testFolder", "./images/testFolder2"];

        let actual_folders = load_filesystem_elements(Path::new("./images"), false).expect("Found empty list!");

        assert_filesystem_elements(&actual_folders, &expected_folders);
    }

    fn assert_filesystem_elements(actual_elements: &Vec<String>, expected_elements: &Vec<&str>) {
        assert_eq!(actual_elements.len(), expected_elements.len());
        for expected in expected_elements {
            assert!(actual_elements.contains(&expected.to_string()));
        }
    }
}