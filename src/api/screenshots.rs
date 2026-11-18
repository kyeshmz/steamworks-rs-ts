use napi_derive::napi;

#[napi]
pub mod screenshots {
    use napi::bindgen_prelude::Error;
    use std::path::Path;

    /// Asks the Steam overlay to capture the current frame.
    #[napi]
    pub fn trigger_screenshot() {
        crate::client::get_client()
            .screenshots()
            .trigger_screenshot();
    }

    /// Imports a JPEG from disk. Pass null for thumbnailFilename to let Steam generate it.
    #[napi(
        ts_args_type = "filename: string, thumbnailFilename: null, width: number, height: number"
    )]
    pub fn add_screenshot_to_library(
        filename: String,
        thumbnail_filename: Option<String>,
        width: i32,
        height: i32,
    ) -> napi::Result<u32> {
        // steamworks-rs fbb7963 drops the thumbnail CString before its FFI call.
        if thumbnail_filename.is_some() {
            return Err(Error::from_reason(
                "Custom screenshot thumbnails are not supported",
            ));
        }
        if width <= 0 || height <= 0 {
            return Err(Error::from_reason("Screenshot dimensions must be positive"));
        }
        crate::client::get_client()
            .screenshots()
            .add_screenshot_to_library(Path::new(&filename), None, width, height)
            .map_err(|e| Error::from_reason(e.to_string()))
    }
}
