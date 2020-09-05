#![cfg(windows)]
mod wmvcore_wrapper;

#[no_mangle]
pub extern "stdcall" fn DllMain(_: u32, _: u32, _: u64) -> u32 {
    1
}
