use napi::bindgen_prelude::{Error, Result};
use std::sync::Mutex;
use steamworks::{AppId, Client, SteamAPIInitError};

lazy_static! {
    static ref STEAM_CLIENT: Mutex<Option<Client>> = Mutex::new(None);
}

pub fn get_client() -> Client {
    let option = STEAM_CLIENT.lock().unwrap().to_owned();
    option.unwrap()
}

pub fn is_initialized() -> bool {
    STEAM_CLIENT.lock().unwrap().is_some()
}

pub fn init(app_id: Option<u32>) -> Result<()> {
    let mut slot = STEAM_CLIENT.lock().unwrap();
    if let Some(client) = slot.as_ref() {
        if let Some(requested) = app_id {
            let active = client.utils().app_id().0;
            if requested != active {
                return Err(Error::from_reason(format!(
                    "Steam is already initialized for App ID {active}; cannot switch to {requested}"
                )));
            }
        }
        return Ok(());
    }
    let client = app_id
        .map(|id| Client::init_app(AppId(id)))
        .unwrap_or_else(Client::init)
        .map_err(|e| match e {
            SteamAPIInitError::FailedGeneric(msg)
            | SteamAPIInitError::NoSteamClient(msg)
            | SteamAPIInitError::VersionMismatch(msg) => Error::from_reason(msg),
        })?;
    *slot = Some(client);
    Ok(())
}
