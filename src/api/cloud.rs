use napi_derive::napi;

#[napi]
pub mod cloud {
    use napi::bindgen_prelude::{BigInt, Buffer, Error};
    use napi::Env;
    use std::io::Read;
    use std::io::Write;

    #[napi]
    pub struct FileInfo {
        pub name: String,
        pub size: BigInt,
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
    pub fn write_file_bytes(env: Env, name: String, data: Buffer) -> Result<napi::JsObject, Error> {
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
