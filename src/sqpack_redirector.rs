use detour::GenericDetour;
use log::debug;
use widestring::{WideCStr, WideCString};

extern "stdcall" {
    fn GetModuleHandleW(lp_module_name: *const u16) -> u64;
    fn GetProcAddress(h_module: u64, lp_proc_name: *const u8) -> u64;
}

type FnCreateFileW = extern "stdcall" fn(*const u16, u32, u32, u64, u32, u32, u64) -> u64;

static mut SQPACK_REDIRECTOR: Option<SqPackRedirector> = None;

pub struct SqPackRedirector {
    create_file_w: GenericDetour<FnCreateFileW>,
}

impl SqPackRedirector {
    pub unsafe fn start() -> detour::Result<()> {
        let kernel32 = GetModuleHandleW(WideCString::from_str("kernel32.dll").unwrap().as_ptr());
        let create_file_w_address = GetProcAddress(kernel32, "CreateFileW".as_ptr());

        let create_file_w = GenericDetour::<FnCreateFileW>::new(
            std::mem::transmute(create_file_w_address),
            Self::hooked_create_file_w,
        )?;
        create_file_w.enable()?;

        let redirector = Self { create_file_w };

        SQPACK_REDIRECTOR.replace(redirector);
        SQPACK_REDIRECTOR.as_mut().unwrap().do_start();

        Ok(())
    }

    fn do_start(&mut self) {}

    #[allow(clippy::too_many_arguments)]
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
            let _self = SQPACK_REDIRECTOR.as_ref().unwrap();
            _self.create_file_w.call(
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
}
