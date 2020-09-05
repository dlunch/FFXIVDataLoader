#![cfg(windows)]

mod hook;
mod wmvcore_wrapper;

pub use wmvcore_wrapper::WMCreateReader;

extern "stdcall" {
    fn AllocConsole();
}

use log::debug;

fn initialize() {
    unsafe { AllocConsole() };
    let _ = pretty_env_logger::formatted_timed_builder()
        .filter_level(log::LevelFilter::Debug)
        .try_init();
    debug!("ffxiv_data_loader init");

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
