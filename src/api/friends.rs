//! Adapted from steamworks.js PRs #211 (Spencer McWilliams) and #186 (Dustin Lacewell).
use napi_derive::napi;

#[napi]
pub mod friends {
    use napi::bindgen_prelude::{BigInt, Buffer, Error};
    use steamworks::{FriendFlags, FriendState, SteamId};

    #[napi(object)]
    pub struct Persona {
        pub name: String,
        pub state: String,
        /// 64 x 64 RGBA bytes when Steam has an avatar available.
        pub medium_avatar: Option<Buffer>,
    }

    fn steam_id(value: BigInt) -> Result<SteamId, Error> {
        match value.words.as_slice() {
            [raw] if !value.sign_bit && *raw != 0 => Ok(SteamId::from_raw(*raw)),
            _ => Err(Error::from_reason(
                "Expected a nonzero unsigned 64-bit Steam ID",
            )),
        }
    }

    /// Returns the IDs of immediate friends, excluding pending invitations.
    #[napi]
    pub fn get_friends() -> Vec<BigInt> {
        crate::client::get_client()
            .friends()
            .get_friends(FriendFlags::IMMEDIATE)
            .into_iter()
            .map(|f| f.id().raw().into())
            .collect()
    }

    #[napi]
    pub fn get_persona_name(steam_id64: BigInt) -> Result<String, Error> {
        let id = steam_id(steam_id64)?;
        Ok(crate::client::get_client().friends().get_friend(id).name())
    }

    #[napi]
    pub enum AvatarSize {
        Small,
        Medium,
        Large,
    }

    /// RGBA pixels at 32x32, 64x64, or 184x184. Returns null when unavailable.
    #[napi(catch_unwind)]
    pub fn get_avatar(steam_id64: BigInt, size: AvatarSize) -> Result<Option<Buffer>, Error> {
        let id = steam_id(steam_id64)?;
        let friend = crate::client::get_client().friends().get_friend(id);
        Ok(match size {
            AvatarSize::Small => friend.small_avatar(),
            AvatarSize::Medium => friend.medium_avatar(),
            AvatarSize::Large => friend.large_avatar(),
        }
        .map(Buffer::from))
    }

    #[napi(catch_unwind)]
    pub fn get_persona(steam_id64: BigInt) -> Result<Persona, Error> {
        let id = steam_id(steam_id64)?;
        let friend = crate::client::get_client().friends().get_friend(id);
        let state = match friend.state() {
            FriendState::Online => "online",
            FriendState::Offline => "offline",
            FriendState::Busy => "busy",
            FriendState::Away => "away",
            FriendState::Snooze => "snooze",
            FriendState::LookingToTrade => "lookingToTrade",
            FriendState::LookingToPlay => "lookingToPlay",
        };
        Ok(Persona {
            name: friend.name(),
            state: state.to_owned(),
            medium_avatar: friend.medium_avatar().map(Buffer::from),
        })
    }

    /// Defaults to name only. True when Steam started a fetch; false when cached.
    #[napi]
    pub fn request_user_information(
        steam_id64: BigInt,
        name_only: Option<bool>,
    ) -> Result<bool, Error> {
        let id = steam_id(steam_id64)?;
        Ok(crate::client::get_client()
            .friends()
            .request_user_information(id, name_only.unwrap_or(true)))
    }
}
