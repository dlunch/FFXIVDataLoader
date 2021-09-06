#![allow(clippy::upper_case_acronyms)]

use std::ffi::CString;

pub type HANDLE = u64;
pub type BOOL = u32;

extern "stdcall" {
    // kernel32
    pub fn GetModuleHandleW(lp_module_name: *const u16) -> HANDLE;
    pub fn LoadLibraryW(lp_lib_file_name: *const u16) -> HANDLE;
    pub fn GetProcAddress(h_module: HANDLE, lp_proc_name: *const i8) -> u64;
    pub fn GetSystemDirectoryW(lp_buffer: *mut u16, u_size: u32) -> u32;
    pub fn AllocConsole() -> BOOL;
}

pub type FnCreateFileW = extern "stdcall" fn(*const u16, u32, u32, u64, u32, u32, u64) -> HANDLE;
pub type FnReadFile = extern "stdcall" fn(HANDLE, *mut u8, u32, *mut u32, u64) -> BOOL;
pub type FnCloseHandle = extern "stdcall" fn(HANDLE) -> BOOL;
pub type FnSetFilePointerEx = extern "stdcall" fn(HANDLE, u64, *mut u64, u32) -> BOOL;
pub type FnDirectInput8Create = extern "stdcall" fn(h_inst: u64, dw_version: u32, riidltf: u64, ppv_out: u64, punk_outer: u64) -> u64;

pub unsafe fn get_proc_address(h_module: HANDLE, proc_name: &str) -> u64 {
    let func_name = CString::new(proc_name).unwrap();
    GetProcAddress(h_module, func_name.as_ptr())
}
