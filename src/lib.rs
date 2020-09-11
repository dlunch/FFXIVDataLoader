#![cfg(windows)]
#![cfg(target_arch = "x86_64")]

mod dinput_wrapper;
mod sqpack_redirector;
mod util;
mod virtual_sqpack;

pub use dinput_wrapper::DirectInput8Create;

extern "stdcall" {
    fn AllocConsole();
}

use std::env;

use log::debug;

fn initialize() {
    unsafe { AllocConsole() };
    let _ = pretty_env_logger::formatted_timed_builder()
        .filter_level(log::LevelFilter::Debug)
        .try_init();
    debug!("ffxiv_data_loader init");

    let mut path = env::current_exe().unwrap();
    path.pop();
    path.push("sqpack");

    unsafe { sqpack_redirector::SqPackRedirector::start(&path).unwrap() };
}

#[no_mangle]
pub extern "stdcall" fn DllMain(_: u32, reason: u32, _: u64) -> u32 {
    if reason == 1 {
        // DLL_PROCESS_ATTACH
        initialize()
    }

    1 // TRUE
}
