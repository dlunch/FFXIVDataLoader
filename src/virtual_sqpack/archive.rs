use std::{
    fs::{self, File},
    io::{self, Read},
    path::Path,
};

use log::debug;

use sqpack::{internal::SqPackIndex, Result, SqPackArchiveId, SqPackFileReference};

use super::{data::VirtualSqPackData, VirtualArchiveFileHandle, VirtualArchiveFileType};

pub struct VirtualSqPackArchive {
    archive_id: SqPackArchiveId,
    index: SqPackIndex,
    dat: VirtualSqPackData,
    dat_index: u32,
}

impl VirtualSqPackArchive {
    pub fn new(sqpack_base_path: &Path, archive_id: SqPackArchiveId) -> io::Result<Self> {
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

        let index_data = fs::read(&index_file_path)?;
        let mut index = SqPackIndex::from_raw(index_data);
        let new_dat_count = index.dat_count() + 1;
        index.write_dat_count(new_dat_count);

        let mut dat0_file = File::open(&dat0_file_path)?;
        let mut dat0_header = vec![0; 0x800];
        dat0_file.read_exact(&mut dat0_header)?;

        let dat = VirtualSqPackData::new(&dat0_header);

        Ok(Self {
            archive_id,
            index,
            dat,
            dat_index: new_dat_count - 1,
        })
    }

    pub fn open(&self, path: &Path) -> Option<VirtualArchiveFileHandle> {
        let extension = path.extension().unwrap().to_str().unwrap();

        if extension == "index" {
            Some((self.archive_id, VirtualArchiveFileType::Index))
        } else if extension.chars().last().unwrap().to_digit(10).unwrap() == self.dat_index {
            Some((self.archive_id, VirtualArchiveFileType::Dat))
        } else {
            None
        }
    }

    pub fn add(&mut self, file_path: &Path, archive_path: &str) -> io::Result<()> {
        let offset = self.dat.write(file_path)?;
        self.write_index(&SqPackFileReference::new(archive_path), offset as u32).unwrap();

        Ok(())
    }

    pub fn read(&self, file_type: &VirtualArchiveFileType, offset: u64, buf: &mut [u8]) -> io::Result<u32> {
        match file_type {
            VirtualArchiveFileType::Index => {
                let data = self.index.data();

                let offset = offset as usize;
                buf.copy_from_slice(&data[offset..offset + buf.len()]);

                Ok(buf.len() as u32)
            }
            VirtualArchiveFileType::Dat => self.dat.read(offset, buf),
        }
    }

    fn write_index(&mut self, reference: &SqPackFileReference, new_offset: u32) -> Result<()> {
        let new_offset = (self.dat_index << 1) | (new_offset >> 3);

        self.index.write_offset(reference.hash.folder, reference.hash.file, new_offset)?;

        Ok(())
    }
}
