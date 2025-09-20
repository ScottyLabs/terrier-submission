use rand::{
    Rng,
    distr::{Alphanumeric, SampleString},
};
use std::fs::File;
use std::{
    fs::Metadata,
    io::Error,
    path::{Path, PathBuf},
};
use zip::{HasZipMetadata, ZipArchive, result::ZipError};

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

pub fn metadata_list(path: &Path) -> Vec<Metadata> {
    let file = File::open(path).unwrap();
    let mut archive = ZipArchive::new(file).unwrap();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
    }
    Vec::new()
}

fn rand_path() -> String {
    Alphanumeric.sample_string(&mut rand::rng(), 20)
}
