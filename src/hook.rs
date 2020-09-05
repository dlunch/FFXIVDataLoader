use detour::GenericDetour;
use dlopen::raw::Library;

type FnCreateFileW = extern "stdcall" fn(u64, u32, u32, u64, u32, u32, u64) -> u64;

extern "stdcall" fn hooked_create_file_w(
    lp_file_name: u64,
    dw_desired_access: u32,
    dw_share_mode: u32,
    lp_security_attributes: u64,
    dw_creation_disposition: u32,
    dw_flags_and_attributes: u32,
    h_template_file: u64,
) -> u64 {
    0
}

pub unsafe fn initialize_hook() -> Result<(), detour::Error> {
    let lib = Library::open("kernel32.dll").unwrap();
    let create_file = lib.symbol("CreateFileW").unwrap();

    let hook = GenericDetour::<FnCreateFileW>::new(create_file, hooked_create_file_w)?;

    hook.enable()?;

    Ok(())
}
