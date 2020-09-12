use std::collections::HashMap;
use std::path::{Path, PathBuf};

use log::debug;
use pathdiff::diff_paths;
use walkdir::WalkDir;

use sqpack::{Result, SqPackArchiveId, SqPackFileReference, SqPackPackage};

struct VirtualSqPackData {
    next_offset: u32,
    files: HashMap<u32, PathBuf>,
}

impl VirtualSqPackData {
    pub fn new() -> Self {
        Self {
            next_offset: 0,
            files: HashMap::new(),
        }
    }

    pub fn write(&mut self, path: &Path) -> u32 {
        0
    }
}

pub struct VirtualSqPack {
    package: SqPackPackage,
    data: HashMap<SqPackArchiveId, VirtualSqPackData>,
}

impl VirtualSqPack {
    pub async fn new(sqpack_path: &Path, data_path: &Path) -> Result<Self> {
        let mut result = Self {
            package: SqPackPackage::new(sqpack_path)?,
            data: HashMap::new(),
        };

        for entry in WalkDir::new(data_path).into_iter().filter_map(std::result::Result::ok) {
            let path = entry.path();
            let relative = diff_paths(path, data_path).unwrap();

            result.add_file(path, relative.as_os_str().to_str().unwrap()).await?
        }

        Ok(result)
    }

    pub fn is_hooked_file(&self, path: &Path) -> bool {
        false
    }

    pub fn read_hooked_file(&self, path: &Path, offset: u64, size: u64) -> Vec<u8> {
        Vec::new()
    }

    async fn add_file(&mut self, file_path: &Path, archive_path: &str) -> Result<()> {
        debug!("Adding {:?}", file_path);

        let archive_id = SqPackArchiveId::from_file_path(archive_path);
        let archive = self.package.archive(archive_id).await?;
        let mut archive = archive.write().await;

        let reference = SqPackFileReference::new(archive_path);

        #[allow(clippy::map_entry)]
        if !self.data.contains_key(&archive_id) {
            let new_dat_count = archive.index.dat_count() + 1;
            archive.index.write_dat_count(new_dat_count);

            self.data.insert(archive_id, VirtualSqPackData::new());
        }

        let dat_index = archive.index.dat_count();
        let dat_offset = self.data.get_mut(&archive_id).unwrap().write(file_path);
        let new_offset = (dat_index << 1) | (dat_offset >> 3);

        archive.index.write_offset(reference.hash.folder, reference.hash.file, new_offset)?;

        Ok(())
    }
}

#[cfg(test)]
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
