use super::localplayer::PlayerSteamId;
use napi_derive::napi;

#[napi]
pub mod apps {
    use super::PlayerSteamId;
    use napi::bindgen_prelude::{BigInt, Error};
    use std::ffi::{CStr, CString};
    use steamworks::{sys, AppId};

    fn with_apps<T>(call: impl FnOnce(*mut sys::ISteamApps) -> T) -> Result<T, Error> {
        let _client = crate::client::get_client();
        let apps = unsafe { sys::SteamAPI_SteamApps_v009() };
        if apps.is_null() {
            return Err(Error::from_reason("Steam apps interface unavailable"));
        }
        Ok(call(apps))
    }

    #[napi(object)]
    pub struct DlcInfo {
        pub app_id: u32,
        pub available: bool,
        pub name: String,
    }

    #[napi]
    pub fn list_dlc() -> Result<Vec<DlcInfo>, Error> {
        with_apps(|apps| unsafe {
            (0..sys::SteamAPI_ISteamApps_GetDLCCount(apps))
                .filter_map(|index| {
                    let (mut app_id, mut available) = (0, false);
                    let mut name = [0u8; 1024];
                    if !sys::SteamAPI_ISteamApps_BGetDLCDataByIndex(
                        apps,
                        index,
                        &mut app_id,
                        &mut available,
                        name.as_mut_ptr().cast(),
                        name.len() as i32,
                    ) {
                        return None;
                    }
                    let len = name.iter().position(|b| *b == 0).unwrap_or(name.len());
                    Some(DlcInfo {
                        app_id,
                        available,
                        name: String::from_utf8_lossy(&name[..len]).into_owned(),
                    })
                })
                .collect()
        })
    }

    #[napi]
    pub fn install_dlc(app_id: u32) -> Result<(), Error> {
        with_apps(|apps| unsafe { sys::SteamAPI_ISteamApps_InstallDLC(apps, app_id) })
    }

    #[napi]
    pub fn uninstall_dlc(app_id: u32) -> Result<(), Error> {
        with_apps(|apps| unsafe { sys::SteamAPI_ISteamApps_UninstallDLC(apps, app_id) })
    }

    #[napi(object)]
    pub struct DlcDownloadProgress {
        pub bytes_downloaded: BigInt,
        pub bytes_total: BigInt,
    }

    #[napi]
    pub fn get_dlc_download_progress(app_id: u32) -> Result<Option<DlcDownloadProgress>, Error> {
        with_apps(|apps| {
            let (mut downloaded, mut total) = (0, 0);
            unsafe {
                sys::SteamAPI_ISteamApps_GetDlcDownloadProgress(
                    apps,
                    app_id,
                    &mut downloaded,
                    &mut total,
                )
            }
            .then(|| DlcDownloadProgress {
                bytes_downloaded: downloaded.into(),
                bytes_total: total.into(),
            })
        })
    }

    #[napi]
    pub fn launch_query_param(key: String) -> Result<String, Error> {
        let key =
            CString::new(key).map_err(|_| Error::from_reason("Launch key contains a NUL byte"))?;
        with_apps(|apps| unsafe {
            let value = sys::SteamAPI_ISteamApps_GetLaunchQueryParam(apps, key.as_ptr());
            if value.is_null() {
                String::new()
            } else {
                CStr::from_ptr(value).to_string_lossy().into_owned()
            }
        })
    }

    #[napi]
    pub fn launch_command_line() -> String {
        crate::client::get_client().apps().launch_command_line()
    }

    #[napi]
    pub fn is_subscribed_app(app_id: u32) -> bool {
        let client = crate::client::get_client();
        client.apps().is_subscribed_app(AppId(app_id))
    }
    #[napi]
    pub fn is_app_installed(app_id: u32) -> bool {
        let client = crate::client::get_client();
        client.apps().is_app_installed(AppId(app_id))
    }

    #[napi]
    pub fn is_dlc_installed(app_id: u32) -> bool {
        let client = crate::client::get_client();
        client.apps().is_dlc_installed(AppId(app_id))
    }

    #[napi]
    pub fn is_subscribed_from_free_weekend() -> bool {
        let client = crate::client::get_client();
        client.apps().is_subscribed_from_free_weekend()
    }

    #[napi]
    pub fn is_vac_banned() -> bool {
        let client = crate::client::get_client();
        client.apps().is_vac_banned()
    }

    #[napi]
    pub fn is_cybercafe() -> bool {
        let client = crate::client::get_client();
        client.apps().is_cybercafe()
    }

    #[napi]
    pub fn is_low_violence() -> bool {
        let client = crate::client::get_client();
        client.apps().is_low_violence()
    }

    #[napi]
    pub fn is_subscribed() -> bool {
        let client = crate::client::get_client();
        client.apps().is_subscribed()
    }

    #[napi]
    pub fn app_build_id() -> i32 {
        let client = crate::client::get_client();
        client.apps().app_build_id()
    }

    #[napi]
    pub fn app_install_dir(app_id: u32) -> String {
        let client = crate::client::get_client();
        client.apps().app_install_dir(AppId(app_id))
    }

    #[napi]
    pub fn app_owner() -> PlayerSteamId {
        let client = crate::client::get_client();
        let steam_id = client.apps().app_owner();
        PlayerSteamId::from_steamid(steam_id)
    }

    #[napi]
    pub fn available_game_languages() -> Vec<String> {
        let client = crate::client::get_client();
        client.apps().available_game_languages()
    }

    #[napi]
    pub fn current_game_language() -> String {
        let client = crate::client::get_client();
        client.apps().current_game_language()
    }

    #[napi]
    pub fn current_beta_name() -> Option<String> {
        let client = crate::client::get_client();
        client.apps().current_beta_name()
    }
}
