#![cfg(windows)]

mod hook;
mod wmvcore_wrapper;

pub use wmvcore_wrapper::WMCreateReader;

fn initialize() {
    win_dbg_logger::init();

    unsafe { hook::initialize_hook().unwrap() };
}

#[no_mangle]
pub extern "stdcall" fn DllMain(_: u32, reason: u32, _: u64) -> u32 {
    if reason == 1 {
        // DLL_PROCESS_ATTACH
        initialize()
    }

    1 // TRUE
}
