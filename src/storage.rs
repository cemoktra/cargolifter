use std::error::Error;
use std::io::{Read, Write};
use std::path::Path;

use crate::tools::crate_name_to_path;

pub trait Storage {
    fn get(
        &self,
        crate_name: &str,
        crate_version: &str,
        mirror: bool,
    ) -> Result<Vec<u8>, Box<dyn Error>>;
    fn put(
        &mut self,
        crate_name: &str,
        crate_version: &str,
        mirror: bool,
        data: &Vec<u8>,
    ) -> Result<(), Box<dyn Error>>;
}

pub struct FileSystemStorage {
    root_folder: String,
}

impl FileSystemStorage {
    pub fn new(root_folder: &str) -> Self {
        Self {
            root_folder: root_folder.into(),
        }
    }
}

impl Storage for FileSystemStorage {
    fn get(
        &self,
        crate_name: &str,
        crate_version: &str,
        mirror: bool,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let root_path = Path::new(&self.root_folder);
        let path = if mirror {
            let mirror_path = root_path.join("mirror");
            mirror_path.join(crate_name_to_path(crate_name))
        } else {
            root_path.join(crate_name_to_path(crate_name))
        };
        let path = path.join(format!("{}", crate_version));
        log::info!("trying to get '{}'", path.to_str().unwrap());

        let mut data = Vec::new();
        let mut file = std::fs::File::open(path)?;
        file.read_to_end(&mut data)?;

        Ok(data)
    }

    fn put(
        &mut self,
        crate_name: &str,
        crate_version: &str,
        mirror: bool,
        data: &Vec<u8>,
    ) -> Result<(), Box<dyn Error>> {
        let root_path = Path::new(&self.root_folder);
        let path = if mirror {
            let mirror_path = root_path.join("mirror");
            mirror_path.join(crate_name_to_path(crate_name))
        } else {
            root_path.join(crate_name_to_path(crate_name))
        };
        std::fs::create_dir_all(path.clone()).unwrap();
        let path = path.join(format!("{}", crate_version));
        log::info!("adding '{}' to storage", path.to_str().unwrap());

        let mut file = std::fs::File::create(path)?;
        file.write(&data)?;

        Ok(())
    }
}
