use napi_derive::napi;

#[napi]
pub mod cloud {
    use napi::bindgen_prelude::{BigInt, Buffer, Error};
    use napi::Env;
    use std::ffi::CString;
    use std::io::Read;
    use std::io::Write;
    use steamworks::sys;

    // The SDK uses ERemoteStoragePlatform as a bit mask. Combinations are not valid
    // discriminants of the Rust enum in the pinned bindings. Use its u32 ABI.
    extern "C" {
        #[link_name = "SteamAPI_ISteamRemoteStorage_GetSyncPlatforms"]
        fn get_platform_mask(
            storage: *mut sys::ISteamRemoteStorage,
            name: *const std::ffi::c_char,
        ) -> u32;
        #[link_name = "SteamAPI_ISteamRemoteStorage_SetSyncPlatforms"]
        fn set_platform_mask(
            storage: *mut sys::ISteamRemoteStorage,
            name: *const std::ffi::c_char,
            mask: u32,
        ) -> bool;
    }

    fn with_storage<T>(f: impl FnOnce(*mut sys::ISteamRemoteStorage) -> T) -> Result<T, Error> {
        let _client = crate::client::get_client();
        let storage = unsafe { sys::SteamAPI_SteamRemoteStorage_v016() };
        if storage.is_null() {
            return Err(Error::from_reason("Steam Cloud is unavailable"));
        }
        Ok(f(storage))
    }

    #[napi]
    pub fn get_sync_platforms(name: String) -> Result<u32, Error> {
        check_name(&name)?;
        let name = CString::new(name).map_err(|e| Error::from_reason(e.to_string()))?;
        with_storage(|storage| unsafe { get_platform_mask(storage, name.as_ptr()) })
    }

    /// Combine Windows (1), macOS (2), PS3 (4), Linux (8), Switch (16),
    /// Android (32) and iOS (64). Use 0 for none or 0xffffffff for all.
    #[napi]
    pub fn set_sync_platforms(name: String, mask: f64) -> Result<bool, Error> {
        check_name(&name)?;
        if mask.fract() != 0.0 || !((0.0..=127.0).contains(&mask) || mask == u32::MAX as f64) {
            return Err(Error::from_reason("Invalid cloud platform mask"));
        }
        let name = CString::new(name).map_err(|e| Error::from_reason(e.to_string()))?;
        with_storage(|storage| unsafe { set_platform_mask(storage, name.as_ptr(), mask as u32) })
    }

    #[napi(object)]
    pub struct CloudQuota {
        pub total_bytes: BigInt,
        pub available_bytes: BigInt,
    }

    #[napi]
    pub fn get_quota() -> Result<Option<CloudQuota>, Error> {
        with_storage(|storage| {
            let (mut total, mut available) = (0, 0);
            unsafe {
                sys::SteamAPI_ISteamRemoteStorage_GetQuota(storage, &mut total, &mut available)
            }
            .then(|| CloudQuota {
                total_bytes: total.into(),
                available_bytes: available.into(),
            })
        })
    }

    /// Hints to Steam that several writes belong together. This is not a transaction.
    #[napi]
    pub fn begin_file_write_batch() -> Result<bool, Error> {
        with_storage(|storage| unsafe {
            sys::SteamAPI_ISteamRemoteStorage_BeginFileWriteBatch(storage)
        })
    }

    /// Call after all writes have settled, even if one failed.
    #[napi]
    pub fn end_file_write_batch() -> Result<bool, Error> {
        with_storage(|storage| unsafe {
            sys::SteamAPI_ISteamRemoteStorage_EndFileWriteBatch(storage)
        })
    }

    #[napi]
    pub struct FileInfo {
        pub name: String,
        size: BigInt,
    }

    #[napi]
    impl FileInfo {
        #[napi(getter)]
        pub fn get_size(&self) -> BigInt {
            self.size.clone()
        }

        #[napi(setter)]
        pub fn set_size(&mut self, value: BigInt) {
            self.size = value;
        }
    }

    #[napi]
    pub fn is_enabled_for_account() -> bool {
        let client = crate::client::get_client();
        client.remote_storage().is_cloud_enabled_for_account()
    }

    #[napi]
    pub fn is_enabled_for_app() -> bool {
        let client = crate::client::get_client();
        client.remote_storage().is_cloud_enabled_for_app()
    }

    #[napi]
    pub fn set_enabled_for_app(enabled: bool) {
        let client = crate::client::get_client();
        client.remote_storage().set_cloud_enabled_for_app(enabled)
    }

    #[napi]
    pub fn read_file(name: String) -> Result<String, Error> {
        let client = crate::client::get_client();
        let mut buf: String = String::new();
        let size = client
            .remote_storage()
            .file(&name)
            .read()
            .read_to_string(&mut buf);

        match size {
            Ok(_) => Ok(buf),
            Err(e) => Err(Error::from_reason(format!("Failed to read file: {}", e))),
        }
    }

    #[napi]
    pub fn write_file(name: String, content: String) -> bool {
        let client = crate::client::get_client();
        let file = client.remote_storage().file(&name);

        let buf = content.as_bytes();
        file.write().write_all(buf).is_ok()
    }

    #[napi]
    pub fn delete_file(name: String) -> bool {
        let client = crate::client::get_client();
        let file = client.remote_storage().file(&name);

        file.delete()
    }

    #[napi]
    pub fn file_exists(name: String) -> bool {
        let client = crate::client::get_client();
        let file = client.remote_storage().file(&name);

        file.exists()
    }

    #[napi]
    pub fn is_file_persisted(name: String) -> bool {
        let client = crate::client::get_client();
        let file = client.remote_storage().file(&name);

        file.is_persisted()
    }

    #[napi]
    pub fn file_timestamp(name: String) -> i64 {
        let client = crate::client::get_client();
        let file = client.remote_storage().file(&name);

        file.timestamp()
    }

    /// Read binary save data without blocking the Node event loop.
    #[napi]
    pub async fn read_file_bytes(name: String) -> Result<Buffer, Error> {
        check_name(&name)?;
        tokio::task::spawn_blocking(move || {
            let file = crate::client::get_client().remote_storage().file(&name);
            if !file.exists() {
                return Err(Error::from_reason("Cloud file does not exist"));
            }
            let mut bytes = Vec::new();
            file.read()
                .read_to_end(&mut bytes)
                .map_err(|e| Error::from_reason(e.to_string()))?;
            Ok(bytes.into())
        })
        .await
        .map_err(|e| Error::from_reason(e.to_string()))?
    }

    #[napi(ts_return_type = "Promise<void>")]
    pub fn write_file_bytes(
        env: &Env,
        name: String,
        data: Buffer,
    ) -> Result<napi::bindgen_prelude::PromiseRaw<'_, ()>, Error> {
        let bytes = data.to_vec();
        env.spawn_future(async move {
            check_name(&name)?;
            tokio::task::spawn_blocking(move || {
                crate::client::get_client()
                    .remote_storage()
                    .file(&name)
                    .write()
                    .write_all(&bytes)
                    .map_err(|e| Error::from_reason(e.to_string()))
            })
            .await
            .map_err(|e| Error::from_reason(e.to_string()))?
        })
    }

    fn check_name(name: &str) -> Result<(), Error> {
        if name.is_empty() || name.contains('\0') {
            return Err(Error::from_reason(
                "Expected a non-empty cloud filename without NUL bytes",
            ));
        }
        Ok(())
    }

    #[napi]
    pub fn list_files() -> Vec<FileInfo> {
        let client = crate::client::get_client();
        client
            .remote_storage()
            .files()
            .into_iter()
            .map(|identity| FileInfo {
                name: identity.name,
                size: BigInt::from(identity.size),
            })
            .collect()
    }
}
