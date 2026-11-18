use napi_derive::napi;

#[napi]
pub mod screenshots {
    use napi::bindgen_prelude::{BigInt, Buffer, Error};
    use std::ffi::CString;
    use std::path::Path;
    use steamworks::sys;

    fn with_screenshots<T>(call: impl FnOnce(*mut sys::ISteamScreenshots) -> T) -> napi::Result<T> {
        let _client = crate::client::get_client();
        let api = unsafe { sys::SteamAPI_SteamScreenshots_v003() };
        if api.is_null() {
            return Err(Error::from_reason("Steam screenshots unavailable"));
        }
        Ok(call(api))
    }

    fn handle(value: u32) -> napi::Result<u32> {
        if value == sys::INVALID_SCREENSHOT_HANDLE {
            Err(Error::from_reason("Steam rejected screenshot"))
        } else {
            Ok(value)
        }
    }

    fn absolute(filename: &str) -> napi::Result<CString> {
        let path = Path::new(filename)
            .canonicalize()
            .map_err(|e| Error::from_reason(e.to_string()))?;
        CString::new(
            path.to_str()
                .ok_or_else(|| Error::from_reason("Screenshot path must be UTF-8"))?,
        )
        .map_err(|e| Error::from_reason(e.to_string()))
    }

    /// Supply tightly packed RGB24 rows, without alpha or stride padding.
    #[napi]
    pub fn write_screenshot(data: Buffer, width: i32, height: i32) -> napi::Result<u32> {
        let size = i64::from(width) * i64::from(height);
        if width <= 0
            || height <= 0
            || size > i64::from(u32::MAX / 3)
            || data.len() != (size * 3) as usize
        {
            return Err(Error::from_reason("Expected width × height × 3 RGB bytes"));
        }
        handle(with_screenshots(|api| unsafe {
            sys::SteamAPI_ISteamScreenshots_WriteScreenshot(
                api,
                data.as_ptr().cast_mut().cast(),
                data.len() as u32,
                width,
                height,
            )
        })?)
    }

    #[napi]
    pub fn hook_screenshots(enabled: bool) {
        crate::client::get_client()
            .screenshots()
            .hook_screenshots(enabled)
    }

    #[napi]
    pub fn set_location(screenshot: u32, location: String) -> napi::Result<bool> {
        let screenshot = handle(screenshot)?;
        let location =
            CString::new(location).map_err(|_| Error::from_reason("Location contains NUL"))?;
        with_screenshots(|api| unsafe {
            sys::SteamAPI_ISteamScreenshots_SetLocation(api, screenshot, location.as_ptr())
        })
    }

    fn tag_id(id: BigInt) -> napi::Result<u64> {
        match id.words.as_slice() {
            [raw] if !id.sign_bit && *raw != 0 => Ok(*raw),
            _ => Err(Error::from_reason("Expected a nonzero unsigned 64-bit ID")),
        }
    }

    #[napi]
    pub fn tag_user(screenshot: u32, steam_id64: BigInt) -> napi::Result<bool> {
        let screenshot = handle(screenshot)?;
        let id = tag_id(steam_id64)?;
        with_screenshots(|api| unsafe {
            sys::SteamAPI_ISteamScreenshots_TagUser(api, screenshot, id)
        })
    }

    #[napi]
    pub fn tag_published_file(screenshot: u32, file_id: BigInt) -> napi::Result<bool> {
        let screenshot = handle(screenshot)?;
        let id = tag_id(file_id)?;
        with_screenshots(|api| unsafe {
            sys::SteamAPI_ISteamScreenshots_TagPublishedFile(api, screenshot, id)
        })
    }

    #[napi]
    pub fn is_screenshots_hooked() -> bool {
        crate::client::get_client()
            .screenshots()
            .is_screenshots_hooked()
    }

    /// Asks the Steam overlay to capture the current frame.
    #[napi]
    pub fn trigger_screenshot() {
        crate::client::get_client()
            .screenshots()
            .trigger_screenshot();
    }

    /// Imports a JPEG from disk. Pass null for thumbnailFilename to let Steam generate it.
    #[napi(
        ts_args_type = "filename: string, thumbnailFilename: string | null, width: number, height: number"
    )]
    pub fn add_screenshot_to_library(
        filename: String,
        thumbnail_filename: Option<String>,
        width: i32,
        height: i32,
    ) -> napi::Result<u32> {
        if width <= 0 || height <= 0 {
            return Err(Error::from_reason("Screenshot dimensions must be positive"));
        }
        let filename = absolute(&filename)?;
        let thumbnail = thumbnail_filename.as_deref().map(absolute).transpose()?;
        handle(with_screenshots(|api| unsafe {
            sys::SteamAPI_ISteamScreenshots_AddScreenshotToLibrary(
                api,
                filename.as_ptr(),
                thumbnail.as_ref().map_or(std::ptr::null(), |s| s.as_ptr()),
                width,
                height,
            )
        })?)
    }
}
