use std::collections::HashMap;
use std::path::{Path, PathBuf};

use async_std::{
    fs::File,
    io::{self, ReadExt},
};
use log::debug;

use sqpack::{internal::SqPackIndex, Result, SqPackArchiveId, SqPackFileReference};

pub struct VirtualSqPackArchive {
    index: SqPackIndex,
    dat_header: Vec<u8>,
    dat_index: u32,
    next_dat_offset: u32,
    files: HashMap<u32, PathBuf>,
}

impl VirtualSqPackArchive {
    pub async fn new(sqpack_base_path: &Path, archive_id: &SqPackArchiveId) -> io::Result<Self> {
        debug!(
            "Creating virtual archive {:02x}{:02x}{:02x}",
            archive_id.root, archive_id.ex, archive_id.part
        );

        let ex_path = if archive_id.ex == 0 {
            "ffxiv".into()
        } else {
            format!("ex{}", archive_id.ex)
        };
        let dat0_file_name = format!("{:02x}{:02x}{:02x}.win32.dat0", archive_id.root, archive_id.ex, archive_id.part);
        let dat0_file_path = sqpack_base_path.join(&ex_path).join(dat0_file_name);

        let index_file_name = format!("{:02x}{:02x}{:02x}.win32.index", archive_id.root, archive_id.ex, archive_id.part);
        let index_file_path = sqpack_base_path.join(&ex_path).join(index_file_name);

        let mut index = SqPackIndex::new(&index_file_path).await?;
        let new_dat_count = index.dat_count() + 1;
        index.write_dat_count(new_dat_count);

        let mut dat0_file = File::open(&dat0_file_path).await.unwrap();
        let mut dat0_header = vec![0; 0x800];
        dat0_file.read(&mut dat0_header).await?;

        Ok(Self {
            index,
            dat_header: dat0_header,
            dat_index: new_dat_count,
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

    pub fn read(&self, offset: u64, buf: &mut [u8]) -> u32 {
        if offset < 0x800 {
            buf.copy_from_slice(&self.dat_header[offset as usize..offset as usize + buf.len()]);

            buf.len() as u32
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
