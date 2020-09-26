use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct VirtualSqPackData {
    header: Vec<u8>,
    next_offset: u32,
    files: HashMap<u32, PathBuf>,
}

impl VirtualSqPackData {
    pub fn new(header_template: Vec<u8>) -> Self {
        let header_len = header_template.len() as u32;
        Self {
            header: header_template,
            next_offset: header_len,
            files: HashMap::new(),
        }
    }

    pub fn write(&mut self, path: &Path) -> u32 {
        let size_on_data = 1000; // TODO

        let offset = self.next_offset;
        self.next_offset += size_on_data;
        self.files.insert(offset, path.into());

        offset
    }

    pub fn read(&self, offset: u64, buf: &mut [u8]) -> u32 {
        if offset < 0x800 {
            buf.copy_from_slice(&self.header[offset as usize..offset as usize + buf.len()]);

            buf.len() as u32
        } else {
            panic!()
        }
    }
}
