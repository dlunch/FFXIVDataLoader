use std::collections::HashMap;
use std::path::{Path, PathBuf};

use async_std::{
    fs::File,
    io::{self, ReadExt},
};
use log::debug;
use pathdiff::diff_paths;
use walkdir::WalkDir;

use sqpack::{internal::SqPackIndex, Result, SqPackArchiveId, SqPackFileReference, SqPackPackage};

struct VirtualSqPackArchive {
    index: SqPackIndex,
    dat_header: Vec<u8>,
    dat_index: u32,
    next_dat_offset: u32,
    files: HashMap<u32, PathBuf>,
}

impl VirtualSqPackArchive {
    pub async fn new(index: SqPackIndex, mut template_file: File, dat_index: u32) -> io::Result<Self> {
        let mut header_template = vec![0; 0x800];
        template_file.read(&mut header_template).await?;

        Ok(Self {
            index,
            dat_header: header_template,
            dat_index,
            next_dat_offset: 0,
            files: HashMap::new(),
        })
    }

    pub fn add_file(&mut self, file_path: &Path, archive_path: &str) -> Result<()> {
        let size_on_data = 1000; // TODO

        let offset = self.next_dat_offset;
        self.next_dat_offset += size_on_data;
        self.files.insert(offset, file_path.into());

        self.write_index(&SqPackFileReference::new(archive_path), offset)
    }

    pub fn read(&self, offset: u64, size: u64) -> &[u8] {
        if offset < 0x800 {
            &self.dat_header[offset as usize..offset as usize + size as usize]
        } else {
            panic!()
        }
    }

    fn write_index(&mut self, reference: &SqPackFileReference, new_offset: u32) -> Result<()> {
        let new_offset = (self.dat_index << 1) | (new_offset >> 3);

        self.index.write_offset(reference.hash.folder, reference.hash.file, new_offset)?;

        Ok(())
    }
}

pub struct VirtualSqPack {
    sqpack_path: PathBuf,
    package: SqPackPackage,
    data: HashMap<SqPackArchiveId, VirtualSqPackArchive>,
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
        let relative_path = diff_paths(path, &self.sqpack_path);

        if relative_path.is_some() {
            let file_name = path.file_name().unwrap().to_str().unwrap();
            let archive_id = SqPackArchiveId::from_sqpack_file_name(file_name);

            return self.data.contains_key(&archive_id);
        }

        false
    }

    pub fn read_hooked_file(&self, path: &Path, offset: u64, buf: &mut [u8]) -> u32 {
        0
    }

    async fn add_file(&mut self, file_path: &Path, archive_path: &str) -> Result<()> {
        debug!("Adding {:?} as {:?}", file_path, archive_path);

        let archive_id = SqPackArchiveId::from_file_path(archive_path);
        let archive = self.package.archive(archive_id).await?;

        #[allow(clippy::map_entry)]
        if !self.data.contains_key(&archive_id) {
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

            let mut index_clone = archive.index.clone();
            let new_dat_count = archive.index.dat_count() + 1;
            index_clone.write_dat_count(new_dat_count);

            self.data.insert(
                archive_id,
                VirtualSqPackArchive::new(index_clone, data_file, new_dat_count).await.unwrap(),
            );
        }

        let virtual_archive = self.data.get_mut(&archive_id).unwrap();
        virtual_archive.add_file(file_path, archive_path)?;

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
