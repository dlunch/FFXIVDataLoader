mod archive;
mod data;

use std::collections::{hash_map::Entry, HashMap};
use std::path::{Path, PathBuf};

use log::debug;
use pathdiff::diff_paths;
use walkdir::WalkDir;

use sqpack::{Result, SqPackArchiveId};

use archive::VirtualSqPackArchive;

pub struct VirtualSqPackPackage {
    sqpack_base_path: PathBuf,
    archives: HashMap<SqPackArchiveId, VirtualSqPackArchive>,
}

impl VirtualSqPackPackage {
    pub async fn new(sqpack_base_path: &Path, data_path: &Path) -> Result<Self> {
        let mut result = Self {
            sqpack_base_path: sqpack_base_path.into(),
            archives: HashMap::new(),
        };

        for entry in WalkDir::new(data_path).into_iter().filter_map(std::result::Result::ok) {
            if entry.file_type().is_file() {
                let path = entry.path();
                let relative = diff_paths(path, data_path).unwrap();

                let archive_path = relative.as_os_str().to_str().unwrap().replace("\\", "/");

                result.add_file(path, &archive_path).await?
            }
        }

        Ok(result)
    }

    pub fn is_virtual_file(&self, path: &Path) -> bool {
        let relative_path = diff_paths(path, &self.sqpack_base_path);

        if relative_path.is_some() {
            let file_name = path.file_name().unwrap().to_str().unwrap();
            let archive_id = SqPackArchiveId::from_sqpack_file_name(file_name);

            let item = self.archives.get(&archive_id);
            if let Some(x) = item {
                return x.is_virtual_file(path);
            }
        }

        false
    }

    pub fn read_virtual_file(&self, path: &Path, offset: u64, buf: &mut [u8]) -> u32 {
        let file_name = path.file_name().unwrap().to_str().unwrap();
        let archive_id = SqPackArchiveId::from_sqpack_file_name(file_name);

        let archive = self.archives.get(&archive_id).unwrap();

        archive.read(path, offset, buf)
    }

    async fn add_file(&mut self, file_path: &Path, archive_path: &str) -> Result<()> {
        debug!("Adding {:?} as {:?}", file_path, archive_path);

        let archive_id = SqPackArchiveId::from_file_path(archive_path);
        let virtual_archive = match self.archives.entry(archive_id) {
            Entry::Occupied(x) => x.into_mut(),
            Entry::Vacant(x) => x.insert(VirtualSqPackArchive::new(&self.sqpack_base_path, &archive_id).await?),
        };

        virtual_archive.add_file(file_path, archive_path)?;

        Ok(())
    }
}

#[cfg(test)]
#[cfg(feature = "test_local")]
mod test {
    use super::*;

    #[async_std::test]
    async fn test_virtual_sqpack() -> Result<()> {
        let _ = pretty_env_logger::formatted_timed_builder()
            .filter_level(log::LevelFilter::Debug)
            .try_init();

        let sqpack_path = Path::new("D:\\games\\SquareEnix\\FINAL FANTASY XIV - A Realm Reborn\\game\\sqpack");
        let data_path = Path::new("D:\\games\\SquareEnix\\FINAL FANTASY XIV - A Realm Reborn\\game\\data");
        let _ = VirtualSqPackPackage::new(sqpack_path, data_path).await?;

        Ok(())
    }
}
