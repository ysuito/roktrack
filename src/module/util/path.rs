//! Path (directories, files) handler.
//!

use std::path::PathBuf;

/// Join paths, return String.
///
pub fn join(paths: &[&str]) -> String {
    let mut path: PathBuf = PathBuf::new();
    for p in paths {
        path.push(p);
    }
    path.into_os_string().into_string().unwrap()
}

pub mod dir {
    //! Provide dir operations.
    //!

    use std::fs;
    use std::path::Path;

    use crate::module::define;

    use super::{RoktrackDir, RoktrackImg, RoktrackPath};

    /// Function to receive a list of paths and create a directory
    ///
    pub fn create_dir_from_path_list(paths: &[&str]) -> Option<String> {
        let path = super::join(paths);
        match fs::create_dir_all(Path::new(&path)) {
            Ok(_) => Some(path),
            Err(_) => None,
        }
    }

    /// This function creates a subdirectory with the given name in one of the two directories.
    /// It uses the first directory if it exists, otherwise it uses the second directory.
    ///
    pub fn create_subdir_in_either_dir(dir1: &str, dir2: &str, name: &str) -> Option<String> {
        // Check if the first directory exists
        let exist: bool = Path::new(dir1).is_dir();
        let parent: &str = match exist {
            true => dir1,
            false => dir2,
        };
        create_dir_from_path_list(&[parent, name])
    }

    /// Create App Data Directory.
    ///
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

    /// Create App Tmp Directiry.
    ///
    pub fn create_tmp_dir() -> String {
        let res = create_dir_from_path_list(&[define::path::EPHEMERAL_DIR, define::system::NAME]);
        match res {
            Some(path) => path,
            None => panic!("Can't Create Tmp Dir."),
        }
    }
    /// Create App Sub Directiry. And Construct Path Config.
    ///
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
/// Retain app path resources
///
#[derive(Debug, Clone)]
pub struct RoktrackPath {
    pub dir: RoktrackDir,
    pub img: RoktrackImg,
}
/// Retain directories for this application.
///
#[derive(Debug, Clone)]
pub struct RoktrackDir {
    pub data: String,
    pub tmp: String,
    pub img: String,
    pub log: String,
}
/// Retain file names for image.
///
#[derive(Debug, Clone)]
pub struct RoktrackImg {
    pub last: String,
    pub crop: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn run_create_dir_from_path_list() {
        dir::create_dir_from_path_list(&["/tmp", "roktracktest", "run_create_dir_from_path_list"]);
        assert!(Path::new("/tmp/roktracktest/run_create_dir_from_path_list").is_dir());
    }

    #[test]
    fn run_create_subdir_in_either_dir() {
        dir::create_subdir_in_either_dir(
            "/tmp/roktracktest1",
            "/tmp/roktracktest",
            "run_create_subdir_in_either_dir",
        );
        assert!(Path::new("/tmp/roktracktest/run_create_subdir_in_either_dir").is_dir());
    }

    #[test]
    fn run_create_data_dir() {
        let res = dir::create_data_dir();
        assert!(Path::new("/data/roktrack").is_dir());
        assert_eq!(res, "/data/roktrack");
    }

    #[test]
    fn run_create_tmp_dir() {
        let res = dir::create_tmp_dir();
        assert!(Path::new("/run/user/1000/roktrack").is_dir());
        assert_eq!(res, "/run/user/1000/roktrack");
    }

    #[test]
    fn run_create_app_sub_dir() {
        let res = dir::create_app_sub_dir();
        assert!(Path::new("/data/roktrack/img").is_dir());
        assert!(Path::new("/data/roktrack/log").is_dir());
        assert_eq!(res.img.last, "/run/user/1000/roktrack/vision.jpg");
        assert_eq!(res.img.crop, "/run/user/1000/roktrack/crop.jpg");
    }
    #[test]
    fn path_join_test() {
        assert_eq!(join(&["/test/", "test"]), "/test/test");
        assert_eq!(join(&["test", "test", "test"]), "test/test/test");
        assert_eq!(join(&["/test/", "test/"]), "/test/test/");
        assert_eq!(
            join(&["./test/", "test/", "test.txt"]),
            "./test/test/test.txt"
        );
    }
}
