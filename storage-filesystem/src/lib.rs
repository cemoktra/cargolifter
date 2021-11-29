use std::io::{Read, Write};
use std::path::Path;

use async_trait::async_trait;

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

#[async_trait]
impl cargolifter_core::Storage for FileSystemStorage {
    async fn get(
        &self,
        crate_name: &str,
        crate_version: &str,
    ) -> Result<Vec<u8>, cargolifter_core::models::StorageError> {
        let root_path = Path::new(&self.root_folder);
        let path = root_path.join(cargolifter_core::get_crate_path(crate_name));
        let path = path.join(crate_version);
        tracing::info!("trying to get '{}'", path.to_str().unwrap());

        let mut data = Vec::new();
        let mut file = std::fs::File::open(path)?;
        file.read_to_end(&mut data)?;

        Ok(data)
    }

    async fn put(
        &mut self,
        crate_name: &str,
        crate_version: &str,
        data: &[u8],
    ) -> Result<(), cargolifter_core::models::StorageError> {
        let root_path = Path::new(&self.root_folder);
        let path = root_path.join(cargolifter_core::get_crate_path(crate_name));
        std::fs::create_dir_all(path.clone()).unwrap();
        let path = path.join(crate_version);
        tracing::info!("adding '{}' to storage", path.to_str().unwrap());

        let mut file = std::fs::File::create(path)?;
        file.write_all(data)?;

        Ok(())
    }
}
