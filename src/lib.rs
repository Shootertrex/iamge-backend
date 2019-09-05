use std::path::Path;

mod filesystem;

pub fn load_folders_and_files(directory: String) {
    filesystem::load_filesystem_elements(Path::new(&directory), true);
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
