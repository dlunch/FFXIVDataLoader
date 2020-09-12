use std::path::{Path, PathBuf};

use log::debug;
use sqpack::{Result, SqPackArchiveId, SqPackFileReference, SqPackPackage};

pub struct VirtualSqPack {
    package: SqPackPackage,
    base_dir: PathBuf,
}

impl VirtualSqPack {
    pub fn new(base_dir: &Path) -> Result<Self> {
        Ok(Self {
            package: SqPackPackage::new(base_dir)?,
            base_dir: base_dir.into(),
        })
    }

    pub async fn add_file(&mut self, path: &str) -> Result<()> {
        debug!("Adding {}", path);

        let archive_id = SqPackArchiveId::from_file_path(path);
        let archive = self.package.archive(archive_id).await?;

        let reference = SqPackFileReference::new(path);

        archive.write().await.index.write_offset(reference.hash.folder, reference.hash.file, 0)?;

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
