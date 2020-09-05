use widestring::WideCString;

extern "stdcall" {
    fn GetSystemDirectoryW(lp_buffer: *mut u16, u_size: u32) -> u32;
    fn LoadLibraryW(lp_lib_file_name: *const u16) -> u64;
    fn GetProcAddress(h_module: u64, lp_proc_name: *const u8) -> u64;
}

type WMCreateReader = extern "stdcall" fn(p_unk_cert: u64, dw_rights: u64, pp_reader: u64) -> u64;

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn WMCreateReader(
    p_unk_cert: u64,
    dw_rights: u64,
    pp_reader: u64,
) -> u64 {
    let mut buffer = Vec::with_capacity(256);
    GetSystemDirectoryW(buffer.as_mut_ptr(), 256);

    let system_path = WideCString::from_vec_with_nul(buffer)
        .unwrap()
        .to_string()
        .unwrap();
    let wmvcore_path = format!("{}\\wmvcore.dll", system_path);

    let wmvcore = LoadLibraryW(WideCString::from_str(wmvcore_path).unwrap().as_ptr());
    let wm_create_reader: WMCreateReader =
        std::mem::transmute(GetProcAddress(wmvcore, "WMCreateReader".as_ptr()));

    wm_create_reader(p_unk_cert, dw_rights, pp_reader)
}
