#![cfg(windows)]
#![cfg(target_arch = "x86_64")]

mod dinput_wrapper;
mod sqpack_redirector;
mod virtual_sqpack;

pub use dinput_wrapper::DirectInput8Create;

extern "stdcall" {
    fn AllocConsole();
}

use std::env;

use async_std::task;
use log::debug;

use virtual_sqpack::VirtualSqPack;

fn initialize() {
    unsafe { AllocConsole() };
    let _ = pretty_env_logger::formatted_timed_builder()
        .filter_level(log::LevelFilter::Debug)
        .try_init();
    debug!("ffxiv_data_loader init");

    let mut base_dir = env::current_exe().unwrap();
    base_dir.pop();

    let mut sqpack_path = base_dir.clone();
    sqpack_path.push("sqpack");

    let mut data_path = base_dir;
    data_path.push("data");

    let virtual_sqpack = task::block_on(async { VirtualSqPack::new(&sqpack_path, &data_path).await.unwrap() });

    unsafe { sqpack_redirector::SqPackRedirector::start(virtual_sqpack).unwrap() };
}

#[no_mangle]
pub extern "stdcall" fn DllMain(_: u32, reason: u32, _: u64) -> u32 {
    if reason == 1 {
        // DLL_PROCESS_ATTACH
        initialize()
    }

    1 // TRUE
}
