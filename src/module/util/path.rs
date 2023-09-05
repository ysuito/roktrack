//! Path Operations Module
//!
//! This module handles path operations for directories and files.

use std::path::PathBuf;

/// Join Paths
///
/// This function takes a slice of strings as input and joins them into a single path string.
/// It uses the PathBuf type to handle platform-specific separators and conversions.
/// It returns the joined path as a String, or panics if the conversion fails.
pub fn join(paths: &[&str]) -> String {
    let mut path: PathBuf = PathBuf::new();
    for p in paths {
        path.push(p);
    }
    path.into_os_string().into_string().unwrap()
}

pub mod dir {
    //! Directory Operations Submodule
    //!
    //! This submodule provides functions for directory operations.

    use std::fs;
    use std::path::Path;

    use super::{RoktrackDir, RoktrackImg, RoktrackPath};
    use crate::module::define;

    /// Create Directory from Path List
    ///
    /// This function takes a slice of strings as input and creates a directory with the joined path.
    /// It uses the `join` function from the parent module to create the path string.
    /// It returns `Some(path)` if the directory creation succeeds, or `None` if it fails.
    pub fn create_dir_from_path_list(paths: &[&str]) -> Option<String> {
        let path = super::join(paths);
        match fs::create_dir_all(Path::new(&path)) {
            Ok(_) => Some(path),
            Err(_) => None,
        }
    }

    /// Create Subdirectory in Either Directory
    ///
    /// This function takes two directory paths and a subdirectory name as input and creates a subdirectory in one of them.
    /// It checks if the first directory exists and uses it as the parent directory if it does.
    /// Otherwise, it uses the second directory as the parent directory.
    /// It returns `Some(path)` if the subdirectory creation succeeds, or `None` if it fails.
    pub fn create_subdir_in_either_dir(dir1: &str, dir2: &str, name: &str) -> Option<String> {
        let exist: bool = Path::new(dir1).is_dir();
        let parent: &str = match exist {
            true => dir1,
            false => dir2,
        };
        create_dir_from_path_list(&[parent, name])
    }

    /// Create Data Directory
    ///
    /// This function creates a data directory for the application.
    /// It uses either `define::path::PERSISTENT_DIR` or `define::path::EPHEMERAL_DIR` as the parent directory,
    /// depending on which one exists.
    /// It uses `define::system::NAME` as the subdirectory name.
    /// It returns the path of the data directory as a String, or panics if it fails to create it.
    pub fn create_data_dir() -> String {
        let res = create_subdir_in_either_dir(
            define::path::PERSISTENT_DIR,
            define::path::EPHEMERAL_DIR,
            define::system::NAME,
        );
        match res {
            Some(path) => path,
            None => panic!("Can't Create Data Dir."),
        }
    }

    /// Create Temporary Directory
    ///
    /// This function creates a temporary directory for the application.
    /// It uses `define::path::EPHEMERAL_DIR` as the parent directory and `define::system::NAME` as the subdirectory name.
    /// It returns the path of the temporary directory as a String, or panics if it fails to create it.
    pub fn create_tmp_dir() -> String {
        let res = create_dir_from_path_list(&[define::path::EPHEMERAL_DIR, define::system::NAME]);
        match res {
            Some(path) => path,
            None => panic!("Can't Create Tmp Dir."),
        }
    }

    /// Create Application Subdirectory and Paths
    ///
    /// This function creates a subdirectory for the application data and constructs a path configuration object.
    /// It uses either `define::path::PERSISTENT_DIR` or `define::path::EPHEMERAL_DIR` as the parent directory,
    /// depending on which one exists.
    /// It uses `define::system::NAME` as the subdirectory name.
    /// It also creates subdirectories for images and logs inside the data directory.
    /// It returns a `RoktrackPath` object that contains the paths of the directories and images as fields.
    pub fn create_app_sub_dir() -> RoktrackPath {
        let data_dir = create_data_dir();
        let tmp_dir = create_tmp_dir();
        let img_dir = create_dir_from_path_list(&[&data_dir, define::path::IMG_DIR]).unwrap();
        let log_dir = create_dir_from_path_list(&[&data_dir, define::path::LOG_DIR]).unwrap();
        let last_img = super::join(&[&tmp_dir, define::path::LAST_IMAGE]);
        let crop_img = super::join(&[&tmp_dir, define::path::CROP_IMAGE]);
        RoktrackPath {
            dir: RoktrackDir {
                data: data_dir,
                tmp: tmp_dir.clone(),
                img: img_dir,
                log: log_dir,
            },
            img: RoktrackImg {
                last: super::join(&[tmp_dir.as_str(), last_img.as_str()]),
                crop: super::join(&[tmp_dir.as_str(), crop_img.as_str()]),
            },
        }
    }
}

/// Paths of Resources
///
/// This struct represents the paths of the resources used by the application.
#[derive(Debug, Clone)]
pub struct RoktrackPath {
    /// Directories Paths
    pub dir: RoktrackDir,
    /// Images Paths
    pub img: RoktrackImg,
}

/// Paths of Directories
///
/// This struct represents the paths of the directories used by the application.
#[derive(Debug, Clone)]
pub struct RoktrackDir {
    /// Data Directory Path
    pub data: String,
    /// Temporary Directory Path
    pub tmp: String,
    /// Image Directory Path
    pub img: String,
    /// Log Directory Path
    pub log: String,
}

/// Paths of Images
///
/// This struct represents the paths of the images used by the application.
#[derive(Debug, Clone)]
pub struct RoktrackImg {
    /// Last Image Path
    pub last: String,
    /// Cropped Image Path
    pub crop: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_create_dir_from_path_list() {
        // Test the create_dir_from_path_list function from the dir submodule
        dir::create_dir_from_path_list(&["/tmp", "roktracktest", "test_create_dir_from_path_list"]);

        // Assert that the directory was created
        assert!(Path::new("/tmp/roktracktest/test_create_dir_from_path_list").is_dir());
    }

    #[test]
    fn test_create_subdir_in_either_dir() {
        // Test the create_subdir_in_either_dir function from the dir submodule
        dir::create_subdir_in_either_dir(
            "/tmp/roktracktest1",
            "/tmp/roktracktest",
            "test_create_subdir_in_either_dir",
        );

        // Assert that the subdirectory was created in one of the parent directories
        assert!(Path::new("/tmp/roktracktest/test_create_subdir_in_either_dir").is_dir());
    }

    #[test]
    fn test_create_data_dir() {
        // Test the create_data_dir function from the dir submodule
        let res = dir::create_data_dir();

        // Assert that the data directory was created
        assert!(Path::new("/data/roktrack").is_dir());

        // Assert that the result matches the expected path
        assert_eq!(res, "/data/roktrack");
    }

    #[test]
    fn test_create_tmp_dir() {
        // Test the create_tmp_dir function from the dir submodule
        let res = dir::create_tmp_dir();

        // Assert that the tmp directory was created
        assert!(Path::new("/run/user/1000/roktrack").is_dir());

        // Assert that the result matches the expected path
        assert_eq!(res, "/run/user/1000/roktrack");
    }

    #[test]
    fn test_create_app_sub_dir() {
        // Test the create_app_sub_dir function from the dir submodule
        let res = dir::create_app_sub_dir();

        // Assert that the img directory was created
        assert!(Path::new("/data/roktrack/img").is_dir());

        // Assert that the log directory was created
        assert!(Path::new("/data/roktrack/log").is_dir());

        // Assert that the last image path matches the expected path
        assert_eq!(res.img.last, "/run/user/1000/roktrack/vision.jpg");

        // Assert that the crop image path matches the expected path
        assert_eq!(res.img.crop, "/run/user/1000/roktrack/crop.jpg");
    }

    #[test]
    fn test_path_join() {
        // Test the join function from the parent module

        // Assert that joining two paths works as expected
        assert_eq!(join(&["/test/", "test"]), "/test/test");

        // Assert that joining three paths works as expected
        assert_eq!(join(&["test", "test", "test"]), "test/test/test");

        // Assert that joining two paths with trailing slashes works as expected
        assert_eq!(join(&["/test/", "test/"]), "/test/test/");

        // Assert that joining relative paths works as expected
        assert_eq!(
            join(&["./test/", "test/", "test.txt"]),
            "./test/test/test.txt"
        );
    }
}
