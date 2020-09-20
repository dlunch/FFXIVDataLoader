#![allow(unused_unsafe)] // clippy bug?

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    slice, stringify,
};

use detour::GenericDetour;
use log::debug;
use widestring::{WideCStr, WideCString};

use crate::virtual_sqpack::VirtualSqPack;

extern "stdcall" {
    fn GetModuleHandleW(lp_module_name: *const u16) -> u64;
    fn GetProcAddress(h_module: u64, lp_proc_name: *const u8) -> u64;
}

type FnCreateFileW = extern "stdcall" fn(*const u16, u32, u32, u64, u32, u32, u64) -> u64;
type FnReadFile = extern "stdcall" fn(u64, *mut u8, u32, *mut u32, u64) -> u64;

static mut SQPACK_REDIRECTOR: Option<SqPackRedirector> = None;

pub struct VirtualFile {
    path: PathBuf,
    offset: u64,
}

pub struct SqPackRedirector {
    virtual_sqpack: VirtualSqPack,
    virtual_file_handles: HashMap<u64, VirtualFile>,
    create_file_w: GenericDetour<FnCreateFileW>,
    read_file: GenericDetour<FnReadFile>,
}

macro_rules! add_hook {
    ($kernel32: expr, $target_fn: ty, $detour: expr) => {{
        let hook_address = GetProcAddress($kernel32, stringify!($target_fn)[2..].as_ptr());
        let hook = GenericDetour::<$target_fn>::new(std::mem::transmute(hook_address), $detour)?;
        hook.enable()?;

        Ok::<_, detour::Error>(hook)
    }};
}

impl SqPackRedirector {
    pub unsafe fn start(virtual_sqpack: VirtualSqPack) -> detour::Result<()> {
        let kernel32 = GetModuleHandleW(WideCString::from_str("kernel32.dll").unwrap().as_ptr());
        let create_file_w = add_hook!(kernel32, FnCreateFileW, Self::hooked_create_file_w)?;
        let read_file = add_hook!(kernel32, FnReadFile, Self::hooked_read_file)?;

        let redirector = Self {
            virtual_sqpack,
            virtual_file_handles: HashMap::new(),
            create_file_w,
            read_file,
        };
        SQPACK_REDIRECTOR.replace(redirector);

        Ok(())
    }

    fn create_virtual_file_handle(&self, path: &Path) -> u64 {
        0
    }

    fn is_virtual_file_handle(&self, handle: u64) -> bool {
        self.virtual_file_handles.contains_key(&handle)
    }

    fn read_virtual_file(&self, handle: u64, buf: &mut [u8]) -> u32 {
        let virtual_file = self.virtual_file_handles.get(&handle).unwrap();

        self.virtual_sqpack.read_hooked_file(&virtual_file.path, virtual_file.offset, buf)
    }

    #[allow(clippy::too_many_arguments)]
    extern "stdcall" fn hooked_create_file_w(
        lp_file_name: *const u16,
        dw_desired_access: u32,
        dw_share_mode: u32,
        lp_security_attributes: u64,
        dw_creation_disposition: u32,
        dw_flags_and_attributes: u32,
        h_template_file: u64,
    ) -> u64 {
        let _self = unsafe { SQPACK_REDIRECTOR.as_ref().unwrap() };
        let path = PathBuf::from(unsafe { WideCStr::from_ptr_str(lp_file_name) }.to_os_string());
        debug!("{:?}", path);

        if _self.virtual_sqpack.is_hooked_file(&path) {
            _self.create_virtual_file_handle(&path)
        } else {
            unsafe {
                _self.create_file_w.call(
                    lp_file_name,
                    dw_desired_access,
                    dw_share_mode,
                    lp_security_attributes,
                    dw_creation_disposition,
                    dw_flags_and_attributes,
                    h_template_file,
                )
            }
        }
    }

    extern "stdcall" fn hooked_read_file(
        h_file: u64,
        lp_buffer: *mut u8,
        n_number_of_bytes_to_read: u32,
        lp_number_of_bytes_read: *mut u32,
        lp_overlapped: u64,
    ) -> u64 {
        let _self = unsafe { SQPACK_REDIRECTOR.as_ref().unwrap() };

        if _self.is_virtual_file_handle(h_file) {
            unsafe {
                let buf = slice::from_raw_parts_mut(lp_buffer, n_number_of_bytes_to_read as usize);
                *lp_number_of_bytes_read = _self.read_virtual_file(h_file, buf);

                1 // TRUE
            }
        } else {
            unsafe {
                _self
                    .read_file
                    .call(h_file, lp_buffer, n_number_of_bytes_to_read, lp_number_of_bytes_read, lp_overlapped)
            }
        }
    }
}
