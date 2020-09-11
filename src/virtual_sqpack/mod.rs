use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
};

use log::debug;
use sqpack::SqPackArchiveId;

use crate::util::cast;

pub struct VirtualSqPack {
    indexes: HashMap<SqPackArchiveId, Vec<u8>>,
    base_dir: PathBuf,
}

impl VirtualSqPack {
    pub fn new(base_dir: &Path) -> Self {
        Self {
            indexes: HashMap::new(),
            base_dir: base_dir.into(),
        }
    }

    pub fn add_file(&mut self, path: &str) -> io::Result<()> {
        let archive_id = SqPackArchiveId::from_file_path(path);
        if !self.indexes.contains_key(&archive_id) {
            self.load_index(&archive_id)?;
        }

        Ok(())
    }

    pub fn is_hooked_file(&self, path: &Path) -> bool {
        false
    }

    pub fn read_hooked_file(&self, path: &Path, offset: u64) -> Vec<u8> {
        Vec::new()
    }

    fn load_index(&mut self, archive_id: &SqPackArchiveId) -> io::Result<()> {
        let index_path = format!(
            "{:02x}{:02x}{:02x}.win32.index",
            archive_id.root, archive_id.ex, archive_id.part
        );
        let base_path = if archive_id.ex == 0 {
            "ffxiv".into()
        } else {
            format!("ex{}", archive_id.ex)
        };

        let mut path = self.base_dir.clone();
        path.push(base_path);
        path.push(index_path);

        debug!("Loading index {:?}", &path);

        let data = fs::read(path)?;

        self.indexes.insert(*archive_id, data);

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_virtual_sqpack() -> io::Result<()> {
        let _ = pretty_env_logger::formatted_timed_builder()
            .filter_level(log::LevelFilter::Debug)
            .try_init();

        let mut virtual_sqpack = VirtualSqPack::new(Path::new(
            "D:\\games\\SquareEnix\\FINAL FANTASY XIV - A Realm Reborn\\game\\sqpack",
        ));

        virtual_sqpack.add_file("common/font1.tex")?;

        Ok(())
    }
}
