use rand::distr::{Alphanumeric, SampleString};
use std::fs::File;
use std::path::{Path, PathBuf};
use zip::{DateTime, ZipArchive, result::ZipError};

pub fn unzip(path: &Path, res_path: Option<&Path>) -> Result<PathBuf, ZipError> {
    let file = File::open(path).unwrap();
    let mut archive = ZipArchive::new(file).unwrap();
    let res_path = match res_path {
        Some(path) => path,
        None => &path.parent().unwrap().join(rand_path()),
    };
    let res = archive.extract(res_path);
    match res {
        Ok(()) => Ok(res_path.to_path_buf()),
        Err(e) => Err(e),
    }
}

pub fn metadata_list(path: &Path) -> Vec<DateTime> {
    let file = File::open(path).unwrap();
    let mut archive = ZipArchive::new(file).unwrap();
    let mut modified_times = Vec::new();
    for i in 0..archive.len() {
        let file = archive.by_index(i).unwrap();
        match file.last_modified() {
            Some(time) => modified_times.push(time),
            None => (),
        };
    }
    modified_times
}

fn rand_path() -> String {
    Alphanumeric.sample_string(&mut rand::rng(), 20)
}
