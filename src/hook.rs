use detour::GenericDetour;
use log::debug;
use widestring::WideCStr;

extern "stdcall" {
    fn CreateFileW(
        lp_file_name: *const u16,
        dw_desired_access: u32,
        dw_share_mode: u32,
        lp_security_attributes: u64,
        dw_creation_disposition: u32,
        dw_flags_and_attributes: u32,
        h_template_file: u64,
    ) -> u64;
}

type FnCreateFileW = extern "stdcall" fn(*const u16, u32, u32, u64, u32, u32, u64) -> u64;

static mut CREATE_FILE_W_HOOK: Option<GenericDetour<FnCreateFileW>> = None;

extern "stdcall" fn hooked_create_file_w(
    lp_file_name: *const u16,
    dw_desired_access: u32,
    dw_share_mode: u32,
    lp_security_attributes: u64,
    dw_creation_disposition: u32,
    dw_flags_and_attributes: u32,
    h_template_file: u64,
) -> u64 {
    let filename = unsafe { WideCStr::from_ptr_str(lp_file_name) };
    debug!("{}", filename.to_string().unwrap());

    unsafe {
        CREATE_FILE_W_HOOK.as_ref().unwrap().call(
            lp_file_name,
            dw_desired_access,
            dw_share_mode,
            lp_security_attributes,
            dw_creation_disposition,
            dw_flags_and_attributes,
            h_template_file,
        )
    }
}

pub unsafe fn initialize_hook() -> Result<(), detour::Error> {
    let create_file_w = std::mem::transmute(CreateFileW as *const u64);

    CREATE_FILE_W_HOOK.replace(GenericDetour::<FnCreateFileW>::new(
        create_file_w,
        hooked_create_file_w,
    )?);

    CREATE_FILE_W_HOOK.as_ref().unwrap().enable()?;

    Ok(())
}
