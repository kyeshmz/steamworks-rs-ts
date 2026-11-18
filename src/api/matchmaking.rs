use napi_derive::napi;

#[napi]
pub mod matchmaking {
    use crate::api::localplayer::PlayerSteamId;
    use napi::bindgen_prelude::{BigInt, Buffer, Error};
    use std::collections::HashMap;
    use std::ffi::{CStr, CString};
    use steamworks::{sys, LobbyId};
    use tokio::sync::oneshot;

    const LOBBY_CHAT_MSG_MAX: usize = 4096;

    #[napi]
    pub enum LobbyType {
        Private,
        FriendsOnly,
        Public,
        Invisible,
    }

    #[napi]
    pub enum LobbyComparison {
        Equal,
        NotEqual,
        GreaterThan,
        GreaterThanOrEqual,
        LessThan,
        LessThanOrEqual,
    }

    impl From<LobbyComparison> for sys::ELobbyComparison {
        fn from(value: LobbyComparison) -> Self {
            match value {
                LobbyComparison::Equal => Self::k_ELobbyComparisonEqual,
                LobbyComparison::NotEqual => Self::k_ELobbyComparisonNotEqual,
                LobbyComparison::GreaterThan => Self::k_ELobbyComparisonGreaterThan,
                LobbyComparison::GreaterThanOrEqual => Self::k_ELobbyComparisonEqualToOrGreaterThan,
                LobbyComparison::LessThan => Self::k_ELobbyComparisonLessThan,
                LobbyComparison::LessThanOrEqual => Self::k_ELobbyComparisonEqualToOrLessThan,
            }
        }
    }

    #[napi]
    pub enum LobbyDistance {
        Close,
        Default,
        Far,
        Worldwide,
    }

    #[napi(object)]
    pub struct LobbyStringFilter {
        pub key: String,
        pub value: String,
        /// Defaults to Equal.
        pub comparison: Option<LobbyComparison>,
    }

    #[napi(object)]
    pub struct LobbyNumberFilter {
        pub key: String,
        pub value: i32,
        pub comparison: LobbyComparison,
    }

    #[napi(object)]
    pub struct LobbyNearFilter {
        pub key: String,
        pub value: i32,
    }

    #[napi(object)]
    pub struct LobbyListFilter {
        pub string_filters: Option<Vec<LobbyStringFilter>>,
        pub number_filters: Option<Vec<LobbyNumberFilter>>,
        /// Sort by proximity to these values, in priority order.
        pub near_value_filters: Option<Vec<LobbyNearFilter>>,
        pub slots_available: Option<u8>,
        pub distance: Option<LobbyDistance>,
        pub count: Option<i32>,
    }

    fn lobby_key(key: String) -> Result<CString, Error> {
        if key.is_empty() || key.len() > sys::k_nMaxLobbyKeyLength as usize {
            return Err(Error::from_reason("Lobby keys must contain 1 to 255 bytes"));
        }
        c_string(key)
    }

    fn c_string(value: String) -> Result<CString, Error> {
        CString::new(value).map_err(|_| Error::from_reason("Unexpected NUL byte in lobby data"))
    }

    fn member_id(value: BigInt) -> Result<u64, Error> {
        match value.words.as_slice() {
            [id] if !value.sign_bit && *id != 0 => Ok(*id),
            _ => Err(Error::from_reason(
                "Expected a nonzero unsigned 64-bit Steam ID",
            )),
        }
    }

    fn raw_matchmaking(_client: &steamworks::Client) -> Result<*mut sys::ISteamMatchmaking, Error> {
        let mm = unsafe { sys::SteamAPI_SteamMatchmaking_v009() };
        if mm.is_null() {
            Err(Error::from_reason("Steam matchmaking unavailable"))
        } else {
            Ok(mm)
        }
    }

    impl LobbyListFilter {
        fn apply(self, client: &steamworks::Client) -> Result<(), Error> {
            // Validate everything before changing Steam's pending filters; they cannot be rolled back.
            let strings = self
                .string_filters
                .unwrap_or_default()
                .into_iter()
                .map(|f| {
                    Ok((
                        lobby_key(f.key)?,
                        c_string(f.value)?,
                        f.comparison.unwrap_or(LobbyComparison::Equal).into(),
                    ))
                })
                .collect::<Result<Vec<_>, Error>>()?;
            let numbers = self
                .number_filters
                .unwrap_or_default()
                .into_iter()
                .map(|f| Ok((lobby_key(f.key)?, f.value, f.comparison.into())))
                .collect::<Result<Vec<_>, Error>>()?;
            let near = self
                .near_value_filters
                .unwrap_or_default()
                .into_iter()
                .map(|f| Ok((lobby_key(f.key)?, f.value)))
                .collect::<Result<Vec<_>, Error>>()?;
            if self.count.is_some_and(|count| count < 0) {
                return Err(Error::from_reason("Lobby result count must be nonnegative"));
            }
            let mm = raw_matchmaking(client)?;
            // fbb7963's string/number/near wrappers pass str.as_ptr() without a terminating NUL.
            unsafe {
                for (key, value, cmp) in strings {
                    sys::SteamAPI_ISteamMatchmaking_AddRequestLobbyListStringFilter(
                        mm,
                        key.as_ptr(),
                        value.as_ptr(),
                        cmp,
                    );
                }
                for (key, value, cmp) in numbers {
                    sys::SteamAPI_ISteamMatchmaking_AddRequestLobbyListNumericalFilter(
                        mm,
                        key.as_ptr(),
                        value,
                        cmp,
                    );
                }
                for (key, value) in near {
                    sys::SteamAPI_ISteamMatchmaking_AddRequestLobbyListNearValueFilter(
                        mm,
                        key.as_ptr(),
                        value,
                    );
                }
            }
            let matchmaking = client.matchmaking();
            if let Some(slots) = self.slots_available {
                matchmaking.set_request_lobby_list_slots_available_filter(slots);
            }
            if let Some(count) = self.count {
                matchmaking.set_request_lobby_list_result_count_filter(count as u64);
            }
            if let Some(distance) = self.distance {
                matchmaking.set_request_lobby_list_distance_filter(match distance {
                    LobbyDistance::Close => steamworks::DistanceFilter::Close,
                    LobbyDistance::Default => steamworks::DistanceFilter::Default,
                    LobbyDistance::Far => steamworks::DistanceFilter::Far,
                    LobbyDistance::Worldwide => steamworks::DistanceFilter::Worldwide,
                });
            }
            Ok(())
        }
    }

    #[napi]
    pub struct Lobby {
        pub id: BigInt,
        lobby_id: LobbyId,
    }

    #[napi]
    impl Lobby {
        #[napi]
        pub async fn join(&self) -> Result<Lobby, Error> {
            join_lobby(self.id.clone()).await
        }

        #[napi]
        pub fn leave(&self) {
            let client = crate::client::get_client();
            client.matchmaking().leave_lobby(self.lobby_id);
        }

        #[napi]
        pub fn open_invite_dialog(&self) {
            let client = crate::client::get_client();
            client.friends().activate_invite_dialog(self.lobby_id);
        }

        /// Broadcast up to 4096 bytes to the lobby, including the sender.
        #[napi]
        pub fn send_chat_message(&self, message: Buffer) -> Result<(), Error> {
            if !(1..=LOBBY_CHAT_MSG_MAX).contains(&message.len()) {
                return Err(Error::from_reason(
                    "Lobby chat messages must contain 1 to 4096 bytes",
                ));
            }
            crate::client::get_client()
                .matchmaking()
                .send_lobby_chat_message(self.lobby_id, &message)
                .map_err(|e| Error::from_reason(e.to_string()))
        }

        /// Read the bytes identified by a LobbyChatMsg callback. Missing entries return an empty buffer.
        #[napi]
        pub fn get_chat_entry(&self, chat_id: i32) -> Buffer {
            let mut buffer = [0u8; LOBBY_CHAT_MSG_MAX];
            crate::client::get_client()
                .matchmaking()
                .get_lobby_chat_entry(self.lobby_id, chat_id, &mut buffer)
                .to_vec()
                .into()
        }

        #[napi]
        pub fn get_member_count(&self) -> usize {
            let client = crate::client::get_client();
            client.matchmaking().lobby_member_count(self.lobby_id)
        }

        #[napi]
        pub fn get_member_limit(&self) -> Option<usize> {
            let client = crate::client::get_client();
            client.matchmaking().lobby_member_limit(self.lobby_id)
        }

        /// Owner only. Returns false if Steam refuses the new limit.
        #[napi]
        pub fn set_member_limit(&self, limit: f64) -> Result<bool, Error> {
            if limit.fract() != 0.0 || !(0.0..=250.0).contains(&limit) {
                return Err(Error::from_reason(
                    "Lobby limit must be an integer from 0 to 250",
                ));
            }
            let client = crate::client::get_client();
            Ok(unsafe {
                sys::SteamAPI_ISteamMatchmaking_SetLobbyMemberLimit(
                    raw_matchmaking(&client)?,
                    self.lobby_id.raw(),
                    limit as i32,
                )
            })
        }

        /// Transfer ownership to a current lobby member. Only the current owner can do this.
        #[napi]
        pub fn set_owner(&self, steam_id64: BigInt) -> Result<bool, Error> {
            let id = member_id(steam_id64)?;
            let client = crate::client::get_client();
            Ok(unsafe {
                sys::SteamAPI_ISteamMatchmaking_SetLobbyOwner(
                    raw_matchmaking(&client)?,
                    self.lobby_id.raw(),
                    id,
                )
            })
        }

        /// Change the lobby's visibility as its owner.
        #[napi]
        pub fn set_type(&self, lobby_type: LobbyType) -> Result<bool, Error> {
            let lobby_type = match lobby_type {
                LobbyType::Private => sys::ELobbyType::k_ELobbyTypePrivate,
                LobbyType::FriendsOnly => sys::ELobbyType::k_ELobbyTypeFriendsOnly,
                LobbyType::Public => sys::ELobbyType::k_ELobbyTypePublic,
                LobbyType::Invisible => sys::ELobbyType::k_ELobbyTypeInvisible,
            };
            let client = crate::client::get_client();
            Ok(unsafe {
                sys::SteamAPI_ISteamMatchmaking_SetLobbyType(
                    raw_matchmaking(&client)?,
                    self.lobby_id.raw(),
                    lobby_type,
                )
            })
        }

        /// Send a lobby invite without opening the overlay. Success does not mean the user joined.
        #[napi]
        pub fn invite_user(&self, steam_id64: BigInt) -> Result<bool, Error> {
            let id = member_id(steam_id64)?;
            let client = crate::client::get_client();
            Ok(unsafe {
                sys::SteamAPI_ISteamMatchmaking_InviteUserToLobby(
                    raw_matchmaking(&client)?,
                    self.lobby_id.raw(),
                    id,
                )
            })
        }

        #[napi]
        pub fn get_members(&self) -> Vec<PlayerSteamId> {
            let client = crate::client::get_client();
            client
                .matchmaking()
                .lobby_members(self.lobby_id)
                .into_iter()
                .map(PlayerSteamId::from_steamid)
                .collect()
        }

        #[napi]
        pub fn get_owner(&self) -> PlayerSteamId {
            let client = crate::client::get_client();
            PlayerSteamId::from_steamid(client.matchmaking().lobby_owner(self.lobby_id))
        }

        /// Set metadata for the local member, not for the lobby itself.
        #[napi]
        pub fn set_member_data(&self, key: String, value: String) -> Result<(), Error> {
            let key = lobby_key(key)?;
            let value = c_string(value)?;
            let client = crate::client::get_client();
            unsafe {
                sys::SteamAPI_ISteamMatchmaking_SetLobbyMemberData(
                    raw_matchmaking(&client)?,
                    self.lobby_id.raw(),
                    key.as_ptr(),
                    value.as_ptr(),
                )
            };
            Ok(())
        }

        /// Steam does not distinguish a missing member value from an empty one.
        #[napi]
        pub fn get_member_data(
            &self,
            steam_id64: BigInt,
            key: String,
        ) -> Result<Option<String>, Error> {
            let key = lobby_key(key)?;
            let (_, id, lossless) = steam_id64.get_u64();
            if !lossless {
                return Err(Error::from_reason(
                    "steamId64 must be an unsigned 64-bit integer",
                ));
            }
            let client = crate::client::get_client();
            unsafe {
                let ptr = sys::SteamAPI_ISteamMatchmaking_GetLobbyMemberData(
                    raw_matchmaking(&client)?,
                    self.lobby_id.raw(),
                    id,
                    key.as_ptr(),
                );
                if ptr.is_null() {
                    return Ok(None);
                }
                let value = CStr::from_ptr(ptr);
                Ok((!value.to_bytes().is_empty()).then(|| value.to_string_lossy().into_owned()))
            }
        }

        #[napi]
        pub fn set_joinable(&self, joinable: bool) -> bool {
            let client = crate::client::get_client();
            client
                .matchmaking()
                .set_lobby_joinable(self.lobby_id, joinable)
        }

        #[napi]
        pub fn get_data(&self, key: String) -> Option<String> {
            let client = crate::client::get_client();
            client
                .matchmaking()
                .lobby_data(self.lobby_id, &key)
                .map(|s| s.to_string())
        }

        #[napi]
        pub fn set_data(&self, key: String, value: String) -> bool {
            let client = crate::client::get_client();
            client
                .matchmaking()
                .set_lobby_data(self.lobby_id, &key, &value)
        }

        #[napi]
        pub fn delete_data(&self, key: String) -> bool {
            let client = crate::client::get_client();
            client.matchmaking().delete_lobby_data(self.lobby_id, &key)
        }

        /// Get an object containing all the lobby data
        #[napi]
        pub fn get_full_data(&self) -> HashMap<String, String> {
            let client = crate::client::get_client();

            let mut data = HashMap::new();

            let count = client.matchmaking().lobby_data_count(self.lobby_id);
            for i in 0..count {
                let maybe_lobby_data = client.matchmaking().lobby_data_by_index(self.lobby_id, i);

                if let Some((key, value)) = maybe_lobby_data {
                    data.insert(key, value);
                }
            }

            data
        }

        /// Merge current lobby data with provided data in a single batch
        /// @returns true if all data was set successfully
        #[napi]
        pub fn merge_full_data(&self, data: HashMap<String, String>) -> bool {
            let matchmaking = crate::client::get_client().matchmaking();
            data.iter()
                .map(|(key, value)| matchmaking.set_lobby_data(self.lobby_id, key, value))
                .all(|x| x)
        }
    }

    #[napi]
    pub async fn create_lobby(lobby_type: LobbyType, max_members: u32) -> Result<Lobby, Error> {
        let client = crate::client::get_client();

        let (tx, rx) = oneshot::channel();

        client.matchmaking().create_lobby(
            match lobby_type {
                LobbyType::Private => steamworks::LobbyType::Private,
                LobbyType::FriendsOnly => steamworks::LobbyType::FriendsOnly,
                LobbyType::Public => steamworks::LobbyType::Public,
                LobbyType::Invisible => steamworks::LobbyType::Invisible,
            },
            max_members,
            |result| {
                tx.send(result).unwrap();
            },
        );

        rx.await
            .unwrap()
            .map(|lobby_id| Lobby {
                id: BigInt::from(lobby_id.raw()),
                lobby_id,
            })
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn join_lobby(lobby_id: BigInt) -> Result<Lobby, Error> {
        let client = crate::client::get_client();

        let (tx, rx) = oneshot::channel();

        client.matchmaking().join_lobby(
            steamworks::LobbyId::from_raw(lobby_id.get_u64().1),
            |result| {
                tx.send(result).unwrap();
            },
        );

        rx.await
            .unwrap()
            .map(|lobby_id| Lobby {
                id: BigInt::from(lobby_id.raw()),
                lobby_id,
            })
            .map_err(|_| Error::from_reason("Failed to join lobby".to_string()))
    }

    #[napi]
    pub async fn get_lobbies(filter: Option<LobbyListFilter>) -> Result<Vec<Lobby>, Error> {
        // RequestLobbyList cancels the preceding search if its callback hasn't fired yet.
        static SEARCH: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
        let _search = SEARCH.lock().await;
        let client = crate::client::get_client();
        if let Some(filter) = filter {
            filter.apply(&client)?;
        }

        let (tx, rx) = oneshot::channel();

        client.matchmaking().request_lobby_list(move |lobbies| {
            let _ = tx.send(lobbies);
        });

        tokio::time::timeout(std::time::Duration::from_secs(30), rx)
            .await
            .map_err(|_| Error::from_reason("Timed out waiting for Steam lobby search"))?
            .map_err(|_| Error::from_reason("Lobby search callback dropped"))?
            .map(|lobbies| {
                lobbies
                    .iter()
                    .map(|lobby_id| Lobby {
                        id: BigInt::from(lobby_id.raw()),
                        lobby_id: *lobby_id,
                    })
                    .collect()
            })
            .map_err(|e| Error::from_reason(e.to_string()))
    }
}
