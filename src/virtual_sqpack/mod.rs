use std::collections::HashMap;
use std::path::Path;

use log::debug;
use sqpack::{Result, SqPackArchiveId, SqPackFileReference, SqPackPackage};

struct VirtualSqPackData {}

impl VirtualSqPackData {
    pub fn new() -> Self {
        Self {}
    }

    pub fn write(&mut self, path: &str) -> u32 {
        0
    }
}

pub struct VirtualSqPack {
    package: SqPackPackage,
    data: HashMap<SqPackArchiveId, VirtualSqPackData>,
}

impl VirtualSqPack {
    pub fn new(base_dir: &Path) -> Result<Self> {
        Ok(Self {
            package: SqPackPackage::new(base_dir)?,
            data: HashMap::new(),
        })
    }

    pub async fn add_file(&mut self, path: &str) -> Result<()> {
        debug!("Adding {}", path);

        let archive_id = SqPackArchiveId::from_file_path(path);
        let archive = self.package.archive(archive_id).await?;
        let mut archive = archive.write().await;

        let reference = SqPackFileReference::new(path);

        #[allow(clippy::map_entry)]
        if !self.data.contains_key(&archive_id) {
            let new_dat_count = archive.index.dat_count() + 1;
            archive.index.write_dat_count(new_dat_count);

            self.data.insert(archive_id, VirtualSqPackData::new());
        }

        let dat_index = archive.index.dat_count();
        let dat_offset = self.data.get_mut(&archive_id).unwrap().write(path);
        let new_offset = (dat_index << 1) | (dat_offset >> 3);

        archive.index.write_offset(reference.hash.folder, reference.hash.file, new_offset)?;

        Ok(())
    }

    pub fn is_hooked_file(&self, path: &Path) -> bool {
        false
    }

    pub fn read_hooked_file(&self, path: &Path, offset: u64, size: u64) -> Vec<u8> {
        Vec::new()
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

        let mut virtual_sqpack = VirtualSqPack::new(Path::new("D:\\games\\SquareEnix\\FINAL FANTASY XIV - A Realm Reborn\\game\\sqpack"))?;

        virtual_sqpack.add_file("common/font1.tex").await?;

        Ok(())
    }
}
