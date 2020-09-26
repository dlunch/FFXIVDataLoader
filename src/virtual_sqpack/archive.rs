use std::path::Path;

use async_std::{
    fs::File,
    io::{self, ReadExt},
};
use log::debug;

use sqpack::{internal::SqPackIndex, Result, SqPackArchiveId, SqPackFileReference};

use super::data::VirtualSqPackData;

pub struct VirtualSqPackArchive {
    index: SqPackIndex,
    dat: VirtualSqPackData,
    dat_index: u32,
}

impl VirtualSqPackArchive {
    pub async fn new(sqpack_base_path: &Path, archive_id: &SqPackArchiveId) -> io::Result<Self> {
        let archive_name = format!("{:02x}{:02x}{:02x}", archive_id.root, archive_id.ex, archive_id.part);
        debug!("Creating virtual archive {}", archive_name);

        let ex_path = if archive_id.ex == 0 {
            "ffxiv".into()
        } else {
            format!("ex{}", archive_id.ex)
        };
        let dat0_file_name = format!("{}.win32.dat0", archive_name);
        let dat0_file_path = sqpack_base_path.join(&ex_path).join(dat0_file_name);

        let index_file_name = format!("{}.win32.index", archive_name);
        let index_file_path = sqpack_base_path.join(&ex_path).join(index_file_name);

        let mut index = SqPackIndex::new(&index_file_path).await?;
        let new_dat_count = index.dat_count() + 1;
        index.write_dat_count(new_dat_count);

        let mut dat0_file = File::open(&dat0_file_path).await?;
        let mut dat0_header = vec![0; 0x800];
        dat0_file.read(&mut dat0_header).await?;

        let dat = VirtualSqPackData::new(dat0_header);

        Ok(Self {
            index,
            dat,
            dat_index: new_dat_count,
        })
    }

    pub fn add_file(&mut self, file_path: &Path, archive_path: &str) -> Result<()> {
        let offset = self.dat.write(file_path);

        self.write_index(&SqPackFileReference::new(archive_path), offset)
    }

    pub fn read(&self, path: &Path, offset: u64, buf: &mut [u8]) -> u32 {
        if path.extension().unwrap().to_str().unwrap() == "index" {
            let data = self.index.data();

            let offset = offset as usize;
            buf.copy_from_slice(&data[offset..offset + buf.len()]);

            buf.len() as u32
        } else {
            self.dat.read(offset, buf)
        }
    }

    fn write_index(&mut self, reference: &SqPackFileReference, new_offset: u32) -> Result<()> {
        let new_offset = (self.dat_index << 1) | (new_offset >> 3);

        self.index.write_offset(reference.hash.folder, reference.hash.file, new_offset)?;

        Ok(())
    }
}
