use napi::bindgen_prelude::BigInt;
use napi_derive::napi;
use steamworks::SteamId;

#[derive(Debug)]
#[napi(object)]
pub struct PlayerSteamId {
    pub steam_id64: BigInt,
    pub steam_id32: String,
    pub account_id: u32,
}

impl PlayerSteamId {
    pub(crate) fn from_steamid(steam_id: SteamId) -> Self {
        Self {
            steam_id64: steam_id.raw().into(),
            steam_id32: steam_id.steamid32(),
            account_id: steam_id.account_id().raw(),
        }
    }
}

#[napi]
pub mod localplayer {
    use super::PlayerSteamId;
    use napi::bindgen_prelude::{BigInt, Error};

    #[napi(object)]
    pub struct SteamIdDetails {
        pub universe: u32,
        pub account_type: u32,
        pub is_invalid: bool,
    }

    #[napi]
    pub fn get_steam_id_details(steam_id: BigInt) -> Result<SteamIdDetails, Error> {
        let (negative, id, lossless) = steam_id.get_u64();
        if negative || !lossless {
            return Err(Error::from_reason(
                "Steam ID must be an unsigned 64-bit integer",
            ));
        }
        let universe = match id >> 56 {
            value @ 1..=4 => value as u32,
            _ => 0,
        };
        let id = steamworks::SteamId::from_raw(id);
        Ok(SteamIdDetails {
            universe,
            account_type: id.account_type() as u32,
            is_invalid: id.is_invalid(),
        })
    }

    #[napi]
    pub enum UserHasLicense {
        HasLicense,
        DoesNotHaveLicense,
        NoAuth,
    }

    #[napi]
    pub fn user_has_license_for_app(
        steam_id: BigInt,
        app_id: u32,
    ) -> Result<UserHasLicense, Error> {
        let (negative, id, lossless) = steam_id.get_u64();
        if negative || !lossless {
            return Err(Error::from_reason(
                "Steam ID must be an unsigned 64-bit integer",
            ));
        }
        let result = crate::client::get_client()
            .user()
            .user_has_license_for_app(steamworks::SteamId::from_raw(id), steamworks::AppId(app_id));
        Ok(match result {
            steamworks::UserHasLicense::HasLicense => UserHasLicense::HasLicense,
            steamworks::UserHasLicense::DoesNotHaveLicense => UserHasLicense::DoesNotHaveLicense,
            steamworks::UserHasLicense::NoAuth => UserHasLicense::NoAuth,
        })
    }

    #[napi]
    pub fn get_steam_id() -> PlayerSteamId {
        let client = crate::client::get_client();
        let steam_id = client.user().steam_id();
        PlayerSteamId::from_steamid(steam_id)
    }

    #[napi]
    pub fn get_name() -> String {
        let client = crate::client::get_client();
        client.friends().name()
    }

    #[napi]
    pub fn get_level() -> u32 {
        let client = crate::client::get_client();
        client.user().level()
    }

    #[napi]
    pub fn is_logged_on() -> bool {
        crate::client::get_client().user().logged_on()
    }

    #[napi]
    pub fn clear_rich_presence() {
        crate::client::get_client().friends().clear_rich_presence();
    }

    #[napi]
    pub fn get_ip_country() -> String {
        let client = crate::client::get_client();
        client.utils().ip_country()
    }

    #[napi]
    pub fn set_rich_presence(key: String, value: Option<String>) {
        let client = crate::client::get_client();
        client.friends().set_rich_presence(&key, value.as_deref());
    }
}
