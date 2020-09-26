#![cfg(windows)]
#![cfg(target_arch = "x86_64")]

mod dinput_wrapper;
mod sqpack_redirector;
mod virtual_sqpack;
mod winapi;

pub use dinput_wrapper::DirectInput8Create;

use std::env;

use async_std::task;
use log::debug;

use virtual_sqpack::VirtualSqPackPackage;

use crate::winapi::AllocConsole;

fn initialize() {
    #[cfg(debug_assertions)]
    {
        unsafe { AllocConsole() };
        let _ = pretty_env_logger::formatted_timed_builder()
            .filter_level(log::LevelFilter::Debug)
            .try_init();
    }
    debug!("ffxiv_data_loader init");

    let base_dir = env::current_exe().unwrap();
    let sqpack_path = base_dir.parent().unwrap().join("sqpack");
    let data_path = base_dir.parent().unwrap().join("data");

    let virtual_sqpack = task::block_on(async { VirtualSqPackPackage::new(&sqpack_path, &data_path).await.unwrap() });

    sqpack_redirector::SqPackRedirector::start(virtual_sqpack).unwrap();
}

#[no_mangle]
pub extern "stdcall" fn DllMain(_: u32, reason: u32, _: u64) -> u32 {
    if reason == 1 {
        // DLL_PROCESS_ATTACH
        initialize()
    }

    1 // TRUE
}
