use std::collections::HashMap;
use std::path::{Path, PathBuf};

use async_std::{
    fs::File,
    io::{self, ReadExt},
};
use log::debug;
use pathdiff::diff_paths;
use walkdir::WalkDir;

use sqpack::{Result, SqPackArchiveId, SqPackFileReference, SqPackPackage};

struct VirtualSqPackData {
    header: Vec<u8>,
    next_offset: u32,
    files: HashMap<u32, PathBuf>,
}

impl VirtualSqPackData {
    pub async fn new(mut template_file: File) -> io::Result<Self> {
        let mut header_template = vec![0; 0x800];
        template_file.read(&mut header_template).await?;

        Ok(Self {
            header: header_template,
            next_offset: 0,
            files: HashMap::new(),
        })
    }

    pub fn write(&mut self, path: &Path) -> u32 {
        0
    }
}

pub struct VirtualSqPack {
    sqpack_path: PathBuf,
    package: SqPackPackage,
    data: HashMap<SqPackArchiveId, VirtualSqPackData>,
}

impl VirtualSqPack {
    pub async fn new(sqpack_path: &Path, data_path: &Path) -> Result<Self> {
        let mut result = Self {
            sqpack_path: sqpack_path.into(),
            package: SqPackPackage::new(sqpack_path)?,
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
        false
    }

    pub fn read_hooked_file(&self, path: &Path, offset: u64, size: u64) -> Vec<u8> {
        Vec::new()
    }

    async fn add_file(&mut self, file_path: &Path, archive_path: &str) -> Result<()> {
        debug!("Adding {:?} as {:?}", file_path, archive_path);

        let archive_id = SqPackArchiveId::from_file_path(archive_path);
        let archive = self.package.archive(archive_id).await?;
        let mut archive = archive.write().await;

        let reference = SqPackFileReference::new(archive_path);

        #[allow(clippy::map_entry)]
        if !self.data.contains_key(&archive_id) {
            let new_dat_count = archive.index.dat_count() + 1;
            archive.index.write_dat_count(new_dat_count);

            let data_file_name = format!("{:02x}{:02x}{:02x}.win32.dat0", archive_id.root, archive_id.ex, archive_id.part);
            let mut data_file_path = self.sqpack_path.clone();
            if archive_id.ex == 0 {
                data_file_path.push("ffxiv");
            } else {
                data_file_path.push(format!("ex{}", archive_id.ex));
            }
            data_file_path.push(data_file_name);

            debug!("Creating data file, template {:?}", data_file_path);
            let data_file = File::open(&data_file_path).await.unwrap();
            self.data.insert(archive_id, VirtualSqPackData::new(data_file).await.unwrap());
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
