#![allow(non_snake_case)]

use dlopen::wrapper::{Container, WrapperApi};
#[derive(dlopen_derive::WrapperApi)]
struct WmvCore {
    WMCreateReader: extern "stdcall" fn(p_unk_cert: u64, dw_rights: u64, pp_reader: u64) -> u64,
}

#[no_mangle]
pub extern "stdcall" fn WMCreateReader(p_unk_cert: u64, dw_rights: u64, pp_reader: u64) -> u64 {
    let cont: Container<WmvCore> = unsafe { Container::load("wmvcore.dll").unwrap() };

    cont.WMCreateReader(p_unk_cert, dw_rights, pp_reader)
}
