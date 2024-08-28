// - Parent
use super::*;

pub(crate) fn is_elevated() -> Result<bool> {
    unsafe {
        let mut token: HANDLE = ptr::null_mut();
        let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut return_length = 0;

        // Open the process token
        if OpenProcessToken(
            winapi::um::processthreadsapi::GetCurrentProcess(),
            TOKEN_QUERY,
            &mut token,
        ) == 0
        {
            return Err(Box::new(io::Error::last_os_error()));
        }

        // Retrieve the elevation status
        let result = GetTokenInformation(
            token,
            TokenElevation,
            &mut elevation as *mut _ as *mut _,
            size_of::<TOKEN_ELEVATION>() as u32,
            &mut return_length,
        );

        CloseHandle(token);

        if result == 0 {
            return Err(Box::new(io::Error::last_os_error()));
        }

        Ok(elevation.TokenIsElevated != 0)
    }
}