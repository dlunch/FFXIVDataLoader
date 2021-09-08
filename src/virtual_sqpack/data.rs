use std::cmp::min;
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

struct SqPackDataFile {
    file: PathBuf,
}

impl SqPackDataFile {
    pub fn new(file: &Path) -> Self {
        Self { file: file.into() }
    }

    pub fn size(&self) -> io::Result<u64> {
        Ok(fs::metadata(&self.file)?.len() + 0x100)
    }

    #[allow(unused_variables)] // TODO
    pub fn read(&self, offset: u64, buf: &mut [u8]) -> io::Result<u64> {
        self.size()
    }
}

enum SqPackDataItem {
    Header(Vec<u8>),
    File(SqPackDataFile),
}

impl SqPackDataItem {
    pub fn read(&self, offset: u64, buf: &mut [u8]) -> io::Result<u64> {
        match self {
            Self::Header(data) => {
                let len = min(data.len() - offset as usize, buf.len());
                buf[..len].copy_from_slice(&data[offset as usize..offset as usize + len]);

                Ok(len as u64)
            }
            Self::File(file) => file.read(offset, buf),
        }
    }

    pub fn size(&self) -> io::Result<u64> {
        match self {
            SqPackDataItem::Header(header) => Ok(header.len() as u64),
            SqPackDataItem::File(file) => file.size(),
        }
    }
}

pub struct VirtualSqPackData {
    data: BTreeMap<u64, SqPackDataItem>,
}

impl VirtualSqPackData {
    pub fn new(header_template: &[u8]) -> Self {
        let mut data = BTreeMap::new();

        let header = SqPackDataItem::Header(header_template.to_vec());
        data.insert(0, header);

        Self { data }
    }

    pub fn write(&mut self, file: &Path) -> io::Result<u64> {
        let last = self.data.iter().last().unwrap();
        let offset = last.0 + last.1.size()?;

        self.data.insert(offset, SqPackDataItem::File(SqPackDataFile::new(file)));

        Ok(offset)
    }

    pub fn read(&self, offset: u64, buf: &mut [u8]) -> io::Result<u32> {
        let mut read_offset = 0;

        for (&k, v) in self.data.range(offset..offset + buf.len() as u64) {
            read_offset += v.read(offset + read_offset - k, &mut buf[read_offset as usize..])?;
        }

        Ok(read_offset as u32)
    }
}
