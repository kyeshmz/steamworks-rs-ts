//! Adapted from steamworks.js PRs #211 (Spencer McWilliams) and #186 (Dustin Lacewell).
use napi_derive::napi;

#[napi]
pub mod friends {
    use napi::bindgen_prelude::{BigInt, Buffer, Error};
    use std::ffi::{CStr, CString};
    use steamworks::{sys, FriendFlags, FriendState, SteamId};

    fn with_friends<T>(f: impl FnOnce(*mut sys::ISteamFriends) -> T) -> Result<T, Error> {
        let _client = crate::client::get_client();
        let friends = unsafe { sys::SteamAPI_SteamFriends_v017() };
        if friends.is_null() {
            return Err(Error::from_reason("Steam friends interface unavailable"));
        }
        Ok(f(friends))
    }

    #[napi]
    pub fn invite_user_to_game(steam_id64: BigInt, connect: String) -> Result<bool, Error> {
        let id = steam_id(steam_id64)?;
        if connect.len() >= sys::k_cchMaxRichPresenceValueLength as usize {
            return Err(Error::from_reason("Connect string exceeds 255 bytes"));
        }
        let connect =
            CString::new(connect).map_err(|_| Error::from_reason("Connect string contains NUL"))?;
        with_friends(|f| unsafe {
            sys::SteamAPI_ISteamFriends_InviteUserToGame(f, id.raw(), connect.as_ptr())
        })
    }

    /// Requests fresh presence. Steam delivers FriendRichPresenceUpdate when available.
    #[napi]
    pub fn request_rich_presence(steam_id64: BigInt) -> Result<(), Error> {
        let id = steam_id(steam_id64)?;
        with_friends(|f| unsafe {
            sys::SteamAPI_ISteamFriends_RequestFriendRichPresence(f, id.raw())
        })
    }

    /// Read a cached presence value. Missing keys return the empty string.
    #[napi]
    pub fn get_rich_presence(steam_id64: BigInt, key: String) -> Result<String, Error> {
        let id = steam_id(steam_id64)?;
        if key.is_empty() || key.len() >= sys::k_cchMaxRichPresenceKeyLength as usize {
            return Err(Error::from_reason(
                "Rich presence keys must contain 1 to 63 bytes",
            ));
        }
        let key =
            CString::new(key).map_err(|_| Error::from_reason("Rich presence key contains NUL"))?;
        with_friends(|f| unsafe {
            let ptr = sys::SteamAPI_ISteamFriends_GetFriendRichPresence(f, id.raw(), key.as_ptr());
            if ptr.is_null() {
                String::new()
            } else {
                CStr::from_ptr(ptr).to_string_lossy().into_owned()
            }
        })
    }

    #[napi(object)]
    pub struct FriendGameInfo {
        pub app_id: u32,
        pub lobby_id: Option<BigInt>,
    }

    #[napi]
    pub fn get_game_played(steam_id64: BigInt) -> Result<Option<FriendGameInfo>, Error> {
        let id = steam_id(steam_id64)?;
        Ok(crate::client::get_client()
            .friends()
            .get_friend(id)
            .game_played()
            .map(|game| FriendGameInfo {
                app_id: game.game.app_id().0,
                lobby_id: (game.lobby.raw() != 0).then(|| game.lobby.raw().into()),
            }))
    }

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

    #[napi(object)]
    pub struct CoplayFriend {
        pub steam_id64: BigInt,
        pub app_id: u32,
        pub last_played: i32,
    }

    #[napi]
    pub fn get_coplay_friends() -> Vec<CoplayFriend> {
        crate::client::get_client()
            .friends()
            .get_coplay_friends()
            .into_iter()
            .map(|friend| CoplayFriend {
                steam_id64: friend.id().raw().into(),
                app_id: friend.coplay_game_played().0,
                last_played: friend.coplay_time(),
            })
            .collect()
    }

    #[napi]
    pub fn set_played_with(steam_id64: BigInt) -> Result<(), Error> {
        let id = steam_id(steam_id64)?;
        crate::client::get_client()
            .friends()
            .get_friend(id)
            .set_played_with();
        Ok(())
    }

    #[napi]
    pub fn get_rich_presence_keys(steam_id64: BigInt) -> Result<Vec<String>, Error> {
        let id = steam_id(steam_id64)?;
        with_friends(|f| unsafe {
            (0..sys::SteamAPI_ISteamFriends_GetFriendRichPresenceKeyCount(f, id.raw()))
                .filter_map(|index| {
                    let key = sys::SteamAPI_ISteamFriends_GetFriendRichPresenceKeyByIndex(
                        f,
                        id.raw(),
                        index,
                    );
                    (!key.is_null()).then(|| CStr::from_ptr(key).to_string_lossy().into_owned())
                })
                .collect()
        })
    }

    #[napi]
    pub enum FriendFilter {
        Immediate,
        Blocked,
        FriendshipRequested,
        RequestingFriendship,
        RequestingInfo,
        Ignored,
        IgnoredFriend,
        ClanMember,
        OnGameServer,
        ChatMember,
        All,
    }

    impl From<FriendFilter> for FriendFlags {
        fn from(value: FriendFilter) -> Self {
            match value {
                FriendFilter::Immediate => Self::IMMEDIATE,
                FriendFilter::Blocked => Self::BLOCKED,
                FriendFilter::FriendshipRequested => Self::FRIENDSHIP_REQUESTED,
                FriendFilter::RequestingFriendship => Self::REQUESTING_FRIENDSHIP,
                FriendFilter::RequestingInfo => Self::REQUESTING_INFO,
                FriendFilter::Ignored => Self::IGNORED,
                FriendFilter::IgnoredFriend => Self::IGNORED_FRIEND,
                FriendFilter::ClanMember => Self::CLAN_MEMBER,
                FriendFilter::OnGameServer => Self::ON_GAME_SERVER,
                FriendFilter::ChatMember => Self::CHAT_MEMBER,
                FriendFilter::All => Self::ALL,
            }
        }
    }

    /// Defaults to immediate friends, excluding pending invitations.
    #[napi]
    pub fn get_friends(filter: Option<FriendFilter>) -> Vec<BigInt> {
        crate::client::get_client()
            .friends()
            .get_friends(filter.unwrap_or(FriendFilter::Immediate).into())
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

    #[napi(object)]
    pub struct FriendsGroup {
        /// Signed 16-bit group ID as Steam reports it. Steam's -1 invalid marker never appears here.
        pub id: i16,
        pub name: String,
    }

    /// Named friends groups of the signed-in user, in Steam's order. Empty when the account has none.
    #[napi]
    pub fn get_friends_groups() -> Result<Vec<FriendsGroup>, Error> {
        with_friends(|f| unsafe {
            let count =
                sys::SteamAPI_ISteamFriends_GetFriendsGroupCount(f).min(sys::k_cFriendsGroupLimit);
            (0..count)
                .filter_map(|index| {
                    let id = sys::SteamAPI_ISteamFriends_GetFriendsGroupIDByIndex(f, index);
                    if id == sys::k_FriendsGroupID_Invalid {
                        return None;
                    }
                    let name = sys::SteamAPI_ISteamFriends_GetFriendsGroupName(f, id);
                    (!name.is_null()).then(|| FriendsGroup {
                        id,
                        name: CStr::from_ptr(name).to_string_lossy().into_owned(),
                    })
                })
                .collect()
        })
    }

    // The addon's ceiling on one group read; Steam documents no per-group member maximum.
    const MAX_GROUP_MEMBERS: i32 = 4096;

    /// Member SteamID64s of one named group. IDs outside the user's groups throw; an empty group returns an empty list.
    #[napi]
    pub fn get_friends_group_members_list(group_id: f64) -> Result<Vec<BigInt>, Error> {
        if group_id.fract() != 0.0
            || !(f64::from(i16::MIN)..=f64::from(i16::MAX)).contains(&group_id)
        {
            return Err(Error::from_reason(
                "Expected a signed 16-bit friends group ID",
            ));
        }
        let group_id = group_id as i16;
        // Steam documents NULL names only for invalid group IDs, so this gates the count call below.
        let name = with_friends(|f| unsafe {
            sys::SteamAPI_ISteamFriends_GetFriendsGroupName(f, group_id)
        })?;
        if name.is_null() {
            return Err(Error::from_reason("Unknown friends group ID"));
        }
        let count = with_friends(|f| unsafe {
            sys::SteamAPI_ISteamFriends_GetFriendsGroupMembersCount(f, group_id)
        })?;
        if count <= 0 {
            return Ok(Vec::new());
        }
        if count > MAX_GROUP_MEMBERS {
            return Err(Error::from_reason(format!(
                "Friends group reports {} members, above the {} read limit",
                count, MAX_GROUP_MEMBERS
            )));
        }
        with_friends(|f| unsafe {
            let mut members = vec![
                sys::CSteamID {
                    m_steamid: sys::CSteamID_SteamID_t { m_unAll64Bits: 0 },
                };
                count as usize
            ];
            sys::SteamAPI_ISteamFriends_GetFriendsGroupMembersList(
                f,
                group_id,
                members.as_mut_ptr(),
                count,
            );
            members
                .iter()
                .filter_map(|member| {
                    let raw = member.m_steamid.m_unAll64Bits;
                    (raw != 0).then(|| raw.into())
                })
                .collect()
        })
    }
}
