#![cfg(windows)]

mod hook;
mod wmvcore_wrapper;

pub use wmvcore_wrapper::WMCreateReader;

#[no_mangle]
pub extern "stdcall" fn DllMain(_: u32, reason: u32, _: u64) -> u32 {
    if reason == 1 {
        // DLL_PROCESS_ATTACH
        unsafe { hook::initialize_hook().unwrap() };
    }

    1 // TRUE
}
