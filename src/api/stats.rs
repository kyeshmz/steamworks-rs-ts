use napi_derive::napi;

#[napi]
pub mod stats {
    use napi::bindgen_prelude::{BigInt, Error};
    use std::ffi::CString;
    use steamworks::sys;
    use tokio::sync::oneshot;

    fn parse_steam_id(steam_id: BigInt) -> Result<u64, Error> {
        let (negative, id, lossless) = steam_id.get_u64();
        if negative || !lossless || id == 0 {
            return Err(Error::from_reason(
                "Steam ID must be a nonzero unsigned 64-bit integer",
            ));
        }
        Ok(id)
    }

    fn user_stats() -> Result<*mut sys::ISteamUserStats, Error> {
        let stats = unsafe { sys::SteamAPI_SteamUserStats_v013() };
        if stats.is_null() {
            return Err(Error::from_reason("Steam user stats interface unavailable"));
        }
        Ok(stats)
    }

    #[napi]
    pub async fn request_global_stats(history_days: f64) -> Result<BigInt, Error> {
        if !history_days.is_finite()
            || history_days.fract() != 0.0
            || !(0.0..=60.0).contains(&history_days)
        {
            return Err(Error::from_reason("History days must be between 0 and 60"));
        }
        let (tx, rx) = oneshot::channel();
        crate::api::runtime::without_dispatch(|| {
            crate::client::get_client()
                .user_stats()
                .request_global_stats(history_days as i32, move |result| {
                    let _ = tx.send(result);
                });
        });
        let game_id = tokio::time::timeout(std::time::Duration::from_secs(30), rx)
            .await
            .map_err(|_| Error::from_reason("Timed out waiting for global stats"))?
            .map_err(|_| Error::from_reason("Global stats request was cancelled"))?
            .map_err(|error| Error::from_reason(error.to_string()))?;
        Ok(game_id.raw().into())
    }

    #[napi]
    pub fn get_global_int(name: String) -> Option<BigInt> {
        if name.contains('\0') {
            return None;
        }
        crate::client::get_client()
            .user_stats()
            .get_global_stat_i64(&name)
            .ok()
            .map(BigInt::from)
    }

    #[napi]
    pub fn get_global_float(name: String) -> Option<f64> {
        if name.contains('\0') {
            return None;
        }
        crate::client::get_client()
            .user_stats()
            .get_global_stat_f64(&name)
            .ok()
    }

    #[napi]
    pub fn get_global_int_history(
        name: String,
        max_days: f64,
    ) -> Result<Option<Vec<BigInt>>, Error> {
        if !max_days.is_finite() || max_days.fract() != 0.0 || !(0.0..=60.0).contains(&max_days) {
            return Err(Error::from_reason("History days must be between 0 and 60"));
        }
        if name.contains('\0') {
            return Ok(None);
        }
        Ok(crate::client::get_client()
            .user_stats()
            .get_global_stat_history_i64(&name, max_days as usize)
            .ok()
            .map(|values| values.into_iter().map(BigInt::from).collect()))
    }

    #[napi]
    pub fn get_global_float_history(
        name: String,
        max_days: f64,
    ) -> Result<Option<Vec<f64>>, Error> {
        if !max_days.is_finite() || max_days.fract() != 0.0 || !(0.0..=60.0).contains(&max_days) {
            return Err(Error::from_reason("History days must be between 0 and 60"));
        }
        if name.contains('\0') {
            return Ok(None);
        }
        Ok(crate::client::get_client()
            .user_stats()
            .get_global_stat_history_f64(&name, max_days as usize)
            .ok())
    }

    #[napi]
    pub fn request_user_stats(steam_id: BigInt) -> Result<(), Error> {
        let id = parse_steam_id(steam_id)?;
        let _client = crate::client::get_client();
        let handle = unsafe { sys::SteamAPI_ISteamUserStats_RequestUserStats(user_stats()?, id) };
        if handle == 0 {
            return Err(Error::from_reason("Steam rejected the user stats request"));
        }
        Ok(())
    }

    #[napi]
    pub fn get_user_stat_int(steam_id: BigInt, name: String) -> Result<Option<i32>, Error> {
        let id = parse_steam_id(steam_id)?;
        let name =
            CString::new(name).map_err(|_| Error::from_reason("Stat name contains a NUL byte"))?;
        let _client = crate::client::get_client();
        let mut value = 0;
        let found = unsafe {
            sys::SteamAPI_ISteamUserStats_GetUserStatInt32(
                user_stats()?,
                id,
                name.as_ptr(),
                &mut value,
            )
        };
        Ok(found.then_some(value))
    }

    #[napi]
    pub fn get_user_stat_float(steam_id: BigInt, name: String) -> Result<Option<f64>, Error> {
        let id = parse_steam_id(steam_id)?;
        let name =
            CString::new(name).map_err(|_| Error::from_reason("Stat name contains a NUL byte"))?;
        let _client = crate::client::get_client();
        let mut value = 0.0;
        let found = unsafe {
            sys::SteamAPI_ISteamUserStats_GetUserStatFloat(
                user_stats()?,
                id,
                name.as_ptr(),
                &mut value,
            )
        };
        Ok(found.then_some(f64::from(value)))
    }

    #[napi]
    pub fn get_user_achievement(steam_id: BigInt, name: String) -> Result<Option<bool>, Error> {
        let id = parse_steam_id(steam_id)?;
        let name = CString::new(name)
            .map_err(|_| Error::from_reason("Achievement name contains a NUL byte"))?;
        let _client = crate::client::get_client();
        let mut value = false;
        let found = unsafe {
            sys::SteamAPI_ISteamUserStats_GetUserAchievement(
                user_stats()?,
                id,
                name.as_ptr(),
                &mut value,
            )
        };
        Ok(found.then_some(value))
    }
    #[napi]
    pub fn get_int(name: String) -> Option<i32> {
        if name.contains('\0') {
            return None;
        }
        let client = crate::client::get_client();
        client.user_stats().get_stat_i32(&name).ok()
    }

    #[napi]
    pub fn set_int(name: String, value: i32) -> bool {
        if name.contains('\0') {
            return false;
        }
        let client = crate::client::get_client();
        client.user_stats().set_stat_i32(&name, value).is_ok()
    }

    #[napi]
    pub fn get_float(name: String) -> Option<f64> {
        if name.contains('\0') {
            return None;
        }
        crate::client::get_client()
            .user_stats()
            .get_stat_f32(&name)
            .ok()
            .map(f64::from)
    }

    #[napi]
    pub fn set_float(name: String, value: f64) -> bool {
        let value = value as f32;
        if !value.is_finite() || name.contains('\0') {
            return false;
        }
        crate::client::get_client()
            .user_stats()
            .set_stat_f32(&name, value)
            .is_ok()
    }

    #[napi]
    pub fn store() -> bool {
        let client = crate::client::get_client();
        client.user_stats().store_stats().is_ok()
    }

    #[napi]
    pub fn reset_all(achievements_too: bool) -> bool {
        let client = crate::client::get_client();
        client
            .user_stats()
            .reset_all_stats(achievements_too)
            .is_ok()
    }
}
