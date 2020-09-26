mod archive;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use log::debug;
use pathdiff::diff_paths;
use walkdir::WalkDir;

use sqpack::{Result, SqPackArchiveId};

use archive::VirtualSqPackArchive;

pub struct VirtualSqPack {
    sqpack_base_path: PathBuf,
    data: HashMap<SqPackArchiveId, VirtualSqPackArchive>,
}

impl VirtualSqPack {
    pub async fn new(sqpack_base_path: &Path, data_path: &Path) -> Result<Self> {
        let mut result = Self {
            sqpack_base_path: sqpack_base_path.into(),
            data: HashMap::new(),
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

    pub fn is_hooked_file(&self, path: &Path) -> bool {
        let relative_path = diff_paths(path, &self.sqpack_base_path);

        if relative_path.is_some() {
            let file_name = path.file_name().unwrap().to_str().unwrap();
            let archive_id = SqPackArchiveId::from_sqpack_file_name(file_name);

            return self.data.contains_key(&archive_id);
        }

        false
    }

    pub fn read_hooked_file(&self, path: &Path, offset: u64, buf: &mut [u8]) -> u32 {
        let file_name = path.file_name().unwrap().to_str().unwrap();
        let archive_id = SqPackArchiveId::from_sqpack_file_name(file_name);

        let data = self.data.get(&archive_id).unwrap();

        data.read(offset, buf)
    }

    async fn add_file(&mut self, file_path: &Path, archive_path: &str) -> Result<()> {
        debug!("Adding {:?} as {:?}", file_path, archive_path);

        let archive_id = SqPackArchiveId::from_file_path(archive_path);
        #[allow(clippy::map_entry)]
        if !self.data.contains_key(&archive_id) {
            self.data
                .insert(archive_id, VirtualSqPackArchive::new(&self.sqpack_base_path, &archive_id).await.unwrap());
        }

        let virtual_archive = self.data.get_mut(&archive_id).unwrap();
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
        let _ = VirtualSqPack::new(sqpack_path, data_path).await?;

        Ok(())
    }
}
