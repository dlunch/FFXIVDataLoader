#![cfg(windows)]
#![cfg(target_arch = "x86_64")]

mod dinput_wrapper;
mod sqpack_redirector;
mod virtual_sqpack;
mod winapi;

pub use dinput_wrapper::DirectInput8Create;

use std::env;
use std::io::{stdin, Read};

use log::debug;

use virtual_sqpack::VirtualSqPackPackage;

use crate::winapi::AllocConsole;

unsafe fn initialize() {
    #[cfg(debug_assertions)]
    {
        AllocConsole();
        let _ = pretty_env_logger::formatted_timed_builder()
            .filter_level(log::LevelFilter::Debug)
            .try_init();

        // ffxiv_dx11.exe restarts itself with reduced privileges, making debugging harder.
        // but it doesn't restart if environment is darwin, so we mock darwin by writing flag value.
        let is_darwin_addr = 0x141BD2405u64 - 0x140000000; // for rev6720372
        let ffxiv_base = crate::winapi::GetModuleHandleW(std::ptr::null());
        let is_darwin_ptr_addr = ffxiv_base + is_darwin_addr;
        let is_darwin_ptr: *mut u8 = std::mem::transmute(is_darwin_ptr_addr);
        *is_darwin_ptr = 1;
    }
    debug!("ffxiv_data_loader init");

    let base_dir = env::current_exe().unwrap();
    let sqpack_path = base_dir.parent().unwrap().join("sqpack");
    let data_path = base_dir.parent().unwrap().join("data");

    let virtual_sqpack = VirtualSqPackPackage::new(&sqpack_path, &data_path).unwrap();

    sqpack_redirector::SqPackRedirector::start(virtual_sqpack).unwrap();
}

unsafe fn uninitialize() {
    #[cfg(debug_assertions)]
    {
        println!("Press enter to exit...");
        let _ = stdin().bytes().next();
    }
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn DllMain(_: u32, reason: u32, _: u64) -> u32 {
    match reason {
        // DLL_PROCESS_ATTACH
        1 => initialize(),
        // DLL_PROCESS_DETACH
        0 => uninitialize(),
        _ => {}
    }

    1 // TRUE
}
