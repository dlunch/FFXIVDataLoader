use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    slice, stringify,
};

use detour::GenericDetour;
use log::{debug, error};
use widestring::{WideCStr, WideCString};

use crate::virtual_sqpack::{VirtualArchiveFileHandle, VirtualSqPackPackage};
use crate::winapi::{get_proc_address, FnCloseHandle, FnCreateFileW, FnReadFile, FnSetFilePointerEx, GetModuleHandleW, BOOL, HANDLE};

static mut SQPACK_REDIRECTOR: Option<SqPackRedirector> = None;

pub struct VirtualFile {
    handle: VirtualArchiveFileHandle,
    offset: u64,
}

pub struct SqPackRedirector {
    virtual_sqpack: VirtualSqPackPackage,
    virtual_file_handles: HashMap<HANDLE, VirtualFile>,
    create_file_w: GenericDetour<FnCreateFileW>,
    read_file: GenericDetour<FnReadFile>,
    close_handle: GenericDetour<FnCloseHandle>,
    set_file_pointer_ex: GenericDetour<FnSetFilePointerEx>,
    handle_sequence: u32,
}

macro_rules! add_hook {
    ($lib: expr, $target_fn: ty, $detour: expr) => {{
        let hook_address = get_proc_address($lib, &stringify!($target_fn)[2..]);
        let hook = GenericDetour::<$target_fn>::new(std::mem::transmute(hook_address), $detour)?;
        hook.enable()?;

        Ok::<_, detour::Error>(hook)
    }};
}

impl SqPackRedirector {
    pub unsafe fn start(virtual_sqpack: VirtualSqPackPackage) -> detour::Result<()> {
        let kernel32 = GetModuleHandleW(WideCString::from_str("kernel32.dll").unwrap().as_ptr());
        let create_file_w = add_hook!(kernel32, FnCreateFileW, Self::hooked_create_file_w)?;
        let read_file = add_hook!(kernel32, FnReadFile, Self::hooked_read_file)?;
        let close_handle = add_hook!(kernel32, FnCloseHandle, Self::hooked_close_handle)?;
        let set_file_pointer_ex = add_hook!(kernel32, FnSetFilePointerEx, Self::hooked_set_file_pointer_ex)?;

        let redirector = Self {
            virtual_sqpack,
            virtual_file_handles: HashMap::new(),
            create_file_w,
            read_file,
            close_handle,
            set_file_pointer_ex,
            handle_sequence: 0,
        };
        SQPACK_REDIRECTOR.replace(redirector);

        Ok(())
    }

    fn open_virtual_file(&mut self, path: &Path) -> Option<HANDLE> {
        if let Some(x) = self.virtual_sqpack.open_virtual_archive_file(path) {
            let sequence = self.handle_sequence;
            self.handle_sequence += 1;

            let handle = (0xFFFF_0000 + sequence) as HANDLE;

            self.virtual_file_handles.insert(handle, VirtualFile { handle: x, offset: 0 });

            debug!("Created virtual file handle: {}, path: {:?}", handle, path);

            Some(handle)
        } else {
            None
        }
    }

    fn is_virtual_file_handle(&self, handle: HANDLE) -> bool {
        self.virtual_file_handles.contains_key(&handle)
    }

    fn read_virtual_file(&self, handle: HANDLE, buf: &mut [u8]) -> u32 {
        let virtual_file = self.virtual_file_handles.get(&handle).unwrap();

        self.virtual_sqpack
            .read_virtual_archive_file(&virtual_file.handle, virtual_file.offset, buf)
    }

    fn close_virtual_file(&mut self, handle: HANDLE) {
        self.virtual_file_handles.remove(&handle);
    }

    fn seek_virtual_file(&mut self, handle: HANDLE, new_offset: u64) {
        self.virtual_file_handles.get_mut(&handle).unwrap().offset = new_offset;
    }

    // detour-rs target doesn't allow unsafe function, so we have to use unsafe block
    #[allow(clippy::too_many_arguments)]
    extern "stdcall" fn hooked_create_file_w(
        lp_file_name: *const u16,
        dw_desired_access: u32,
        dw_share_mode: u32,
        lp_security_attributes: u64,
        dw_creation_disposition: u32,
        dw_flags_and_attributes: u32,
        h_template_file: u64,
    ) -> HANDLE {
        unsafe {
            let _self = SQPACK_REDIRECTOR.as_mut().unwrap();
            let path = PathBuf::from(WideCStr::from_ptr_str(lp_file_name).to_os_string());
            debug!("CreateFile {:?}", path);

            if let Some(x) = _self.open_virtual_file(&path) {
                x
            } else {
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
        h_file: HANDLE,
        lp_buffer: *mut u8,
        n_number_of_bytes_to_read: u32,
        lp_number_of_bytes_read: *mut u32,
        lp_overlapped: u64,
    ) -> BOOL {
        unsafe {
            let _self = SQPACK_REDIRECTOR.as_ref().unwrap();
            debug!("ReadFile {} {}", h_file, n_number_of_bytes_to_read);

            if _self.is_virtual_file_handle(h_file) {
                let buf = slice::from_raw_parts_mut(lp_buffer, n_number_of_bytes_to_read as usize);
                *lp_number_of_bytes_read = _self.read_virtual_file(h_file, buf);

                1 // TRUE
            } else {
                _self
                    .read_file
                    .call(h_file, lp_buffer, n_number_of_bytes_to_read, lp_number_of_bytes_read, lp_overlapped)
            }
        }
    }

    extern "stdcall" fn hooked_close_handle(handle: HANDLE) -> BOOL {
        unsafe {
            let _self = SQPACK_REDIRECTOR.as_mut().unwrap();

            if _self.is_virtual_file_handle(handle) {
                _self.close_virtual_file(handle);

                1 // TRUE
            } else {
                _self.close_handle.call(handle)
            }
        }
    }

    extern "stdcall" fn hooked_set_file_pointer_ex(
        h_file: HANDLE,
        li_distance_to_move: u64,
        lp_new_file_pointer: *mut u64,
        dw_move_method: u32,
    ) -> BOOL {
        unsafe {
            let _self = SQPACK_REDIRECTOR.as_mut().unwrap();
            debug!("SetFilePointerEx {} {}, {}", h_file, li_distance_to_move, dw_move_method);

            if _self.is_virtual_file_handle(h_file) {
                if dw_move_method == 0 {
                    // FILE_BEGIN
                    _self.seek_virtual_file(h_file, li_distance_to_move);

                    if !lp_new_file_pointer.is_null() {
                        *lp_new_file_pointer = li_distance_to_move;
                    }

                    1 // TRUE
                } else {
                    error!("Unsupported SetFilePointerEx MoveMethod {}", dw_move_method);

                    0
                }
            } else {
                _self
                    .set_file_pointer_ex
                    .call(h_file, li_distance_to_move, lp_new_file_pointer, dw_move_method)
            }
        }
    }
}
