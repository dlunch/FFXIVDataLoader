use widestring::WideCString;

use crate::winapi::{get_proc_address, FnDirectInput8Create, GetSystemDirectoryW, LoadLibraryW};

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn DirectInput8Create(h_inst: u64, dw_version: u32, riidltf: u64, ppv_out: u64, punk_outer: u64) -> u64 {
    let mut buffer = vec![0; 256];
    GetSystemDirectoryW(buffer.as_mut_ptr(), 256);

    let system_path = WideCString::from_vec_with_nul(buffer).unwrap().to_string().unwrap();
    let dinput8_path = format!("{}\\dinput8.dll", system_path);

    let dinput8 = LoadLibraryW(WideCString::from_str(dinput8_path).unwrap().as_ptr());
    let direct_input_8_create: FnDirectInput8Create = std::mem::transmute(get_proc_address(dinput8, "DirectInput8Create"));

    direct_input_8_create(h_inst, dw_version, riidltf, ppv_out, punk_outer)
}
