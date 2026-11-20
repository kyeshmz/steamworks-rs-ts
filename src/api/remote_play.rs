use napi_derive::napi;

#[napi]
pub mod remote_play {
    use napi::bindgen_prelude::{BigInt, Error};
    use steamworks::{sys, RemotePlaySessionId, SteamDeviceFormFactor};

    #[napi(object)]
    pub struct RemotePlayResolution {
        pub width: u32,
        pub height: u32,
    }

    #[napi(object)]
    pub struct RemotePlaySession {
        pub id: u32,
        pub user: BigInt,
        pub client_name: String,
        pub client_form_factor: Option<String>,
        pub client_resolution: Option<RemotePlayResolution>,
    }

    #[napi]
    pub fn sessions() -> Result<Vec<u32>, Error> {
        let _client = crate::client::get_client();
        let remote = unsafe { sys::SteamAPI_SteamRemotePlay_v004() };
        if remote.is_null() {
            return Err(Error::from_reason(
                "Steam Remote Play interface unavailable",
            ));
        }
        let count = unsafe { sys::SteamAPI_ISteamRemotePlay_GetSessionCount(remote) };
        if count > 4096 {
            return Err(Error::from_reason(
                "Remote Play session count exceeds limit",
            ));
        }
        Ok((0..count)
            .filter_map(|index| {
                let id =
                    unsafe { sys::SteamAPI_ISteamRemotePlay_GetSessionID(remote, index as i32) };
                (id != 0).then_some(id)
            })
            .collect())
    }

    #[napi]
    pub fn session(id: f64) -> Result<Option<RemotePlaySession>, Error> {
        if !id.is_finite() || id.fract() != 0.0 || id < 1.0 || id > u32::MAX as f64 {
            return Err(Error::from_reason(
                "Remote Play session ID must be a nonzero 32-bit integer",
            ));
        }
        let id = id as u32;
        let client = crate::client::get_client();
        let remote = unsafe { sys::SteamAPI_SteamRemotePlay_v004() };
        if remote.is_null() {
            return Err(Error::from_reason(
                "Steam Remote Play interface unavailable",
            ));
        }
        let play = client.remote_play();
        let session = play.session(RemotePlaySessionId::from_raw(id));
        let Some(client_name) = session.client_name() else {
            return Ok(None);
        };
        let client_form_factor = session.client_form_factor().map(|factor| {
            match factor {
                SteamDeviceFormFactor::Phone => "phone",
                SteamDeviceFormFactor::Tablet => "tablet",
                SteamDeviceFormFactor::Computer => "computer",
                SteamDeviceFormFactor::TV => "tv",
            }
            .to_owned()
        });
        let client_resolution = session.client_resolution().and_then(|(width, height)| {
            (width <= i32::MAX as u32 && height <= i32::MAX as u32)
                .then_some(RemotePlayResolution { width, height })
        });
        Ok(Some(RemotePlaySession {
            id,
            user: session.user().raw().into(),
            client_name,
            client_form_factor,
            client_resolution,
        }))
    }

    #[napi]
    pub fn invite(friend: BigInt) -> Result<bool, Error> {
        let (negative, id, lossless) = friend.get_u64();
        if negative || !lossless || id == 0 {
            return Err(Error::from_reason(
                "Friend Steam ID must be a nonzero unsigned 64-bit bigint",
            ));
        }
        let _client = crate::client::get_client();
        let remote = unsafe { sys::SteamAPI_SteamRemotePlay_v004() };
        if remote.is_null() {
            return Err(Error::from_reason(
                "Steam Remote Play interface unavailable",
            ));
        }
        Ok(unsafe { sys::SteamAPI_ISteamRemotePlay_BSendRemotePlayTogetherInvite(remote, id) })
    }
}
