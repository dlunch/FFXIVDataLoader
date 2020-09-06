use std::path::Path;

pub struct VirtualSqPack {}

impl VirtualSqPack {
    pub fn new(base_dir: &Path) -> Self {
        Self {}
    }

    pub fn is_hooked_file(&self, path: &Path) -> bool {
        false
    }

    pub fn read_hooked_file(&self, path: &Path, offset: u64) -> Vec<u8> {
        Vec::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_virtual_sqpack() {
        let virtual_sqpack = VirtualSqPack::new(Path::new(
            "D:\\games\\SquareEnix\\FINAL FANTASY XIV - A Realm Reborn\\game\\sqpack",
        ));
    }
}
