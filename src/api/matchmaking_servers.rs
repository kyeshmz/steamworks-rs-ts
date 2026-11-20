use napi_derive::napi;

#[napi]
pub mod matchmaking_servers {
    use crate::api::runtime;
    use napi::bindgen_prelude::{BigInt, Error};
    use steamworks::sys;

    const MAX_FILTERS: usize = 32;
    const MAX_INDEX: f64 = 100_000.0;

    unsafe extern "C" {
        fn steamworksjs_server_list_response() -> *mut sys::ISteamMatchmakingServerListResponse;
    }

    #[napi(object)]
    pub struct ServerFilter {
        pub key: String,
        pub value: String,
    }

    #[napi(object)]
    pub struct ServerInfo {
        pub app_id: u32,
        pub steam_id: BigInt,
        pub address: String,
        pub query_port: u32,
        pub connection_port: u32,
        pub ping_ms: i32,
        pub players: i32,
        pub max_players: i32,
        pub bot_players: i32,
        pub version: i32,
        pub last_time_played: u32,
        pub successful_response: bool,
        pub do_not_refresh: bool,
        pub password: bool,
        pub secure: bool,
        pub name: String,
        pub map: String,
        pub game_dir: String,
        pub description: String,
        pub tags: String,
    }

    fn string_from_array(array: &[std::os::raw::c_char]) -> String {
        let bytes: Vec<u8> = array
            .iter()
            .take_while(|c| **c != 0)
            .map(|c| *c as u8)
            .collect();
        String::from_utf8_lossy(&bytes).into_owned()
    }

    #[napi]
    pub struct ServerList {
        interface: usize,
        handle: Option<usize>,
    }

    impl ServerList {
        fn with_handle<T>(
            &self,
            f: impl FnOnce(*mut sys::ISteamMatchmakingServers, sys::HServerListRequest) -> T,
        ) -> Result<T, Error> {
            let handle = self
                .handle
                .ok_or_else(|| Error::from_reason("Server list is released"))?;
            Ok(runtime::without_dispatch(|| {
                f(self.interface as _, handle as _)
            }))
        }

        fn release_inner(&mut self) -> bool {
            if let Some(handle) = self.handle.take() {
                runtime::without_dispatch(|| unsafe {
                    sys::SteamAPI_ISteamMatchmakingServers_ReleaseRequest(
                        self.interface as _,
                        handle as _,
                    );
                });
                true
            } else {
                false
            }
        }
    }

    impl Drop for ServerList {
        fn drop(&mut self) {
            self.release_inner();
        }
    }

    #[napi]
    impl ServerList {
        #[napi]
        pub fn is_refreshing(&self) -> Result<bool, Error> {
            self.with_handle(|interface, handle| unsafe {
                sys::SteamAPI_ISteamMatchmakingServers_IsRefreshing(interface, handle)
            })
        }

        #[napi]
        pub fn count(&self) -> Result<i32, Error> {
            self.with_handle(|interface, handle| unsafe {
                sys::SteamAPI_ISteamMatchmakingServers_GetServerCount(interface, handle)
            })
        }

        #[napi]
        pub fn get_server(&self, index: f64) -> Result<Option<ServerInfo>, Error> {
            if !index.is_finite() || index.fract() != 0.0 || !(0.0..MAX_INDEX).contains(&index) {
                return Err(Error::from_reason(
                    "Server index must be an integer from 0 to 99999",
                ));
            }
            self.with_handle(|interface, handle| unsafe {
                let index = index as i32;
                let count =
                    sys::SteamAPI_ISteamMatchmakingServers_GetServerCount(interface, handle);
                if index >= count {
                    return None;
                }
                let item = sys::SteamAPI_ISteamMatchmakingServers_GetServerDetails(
                    interface, handle, index,
                );
                if item.is_null() {
                    return None;
                }
                let item = *item;
                Some(ServerInfo {
                    app_id: item.m_nAppID,
                    steam_id: BigInt::from(item.m_steamID.m_steamid.m_unAll64Bits),
                    address: std::net::Ipv4Addr::from(item.m_NetAdr.m_unIP).to_string(),
                    query_port: item.m_NetAdr.m_usQueryPort.into(),
                    connection_port: item.m_NetAdr.m_usConnectionPort.into(),
                    ping_ms: item.m_nPing,
                    players: item.m_nPlayers,
                    max_players: item.m_nMaxPlayers,
                    bot_players: item.m_nBotPlayers,
                    version: item.m_nServerVersion,
                    last_time_played: item.m_ulTimeLastPlayed,
                    successful_response: item.m_bHadSuccessfulResponse,
                    do_not_refresh: item.m_bDoNotRefresh,
                    password: item.m_bPassword,
                    secure: item.m_bSecure,
                    name: string_from_array(&item.m_szServerName),
                    map: string_from_array(&item.m_szMap),
                    game_dir: string_from_array(&item.m_szGameDir),
                    description: string_from_array(&item.m_szGameDescription),
                    tags: string_from_array(&item.m_szGameTags),
                })
            })
        }

        #[napi]
        pub fn cancel(&mut self) -> bool {
            self.release_inner()
        }

        #[napi]
        pub fn release(&mut self) -> bool {
            self.release_inner()
        }
    }

    #[napi]
    pub fn request_list(
        app_id: f64,
        #[napi(
            ts_arg_type = "'internet' | 'lan' | 'friends' | 'favorites' | 'history' | 'spectator'"
        )]
        kind: String,
        filters: Option<Vec<ServerFilter>>,
    ) -> Result<ServerList, Error> {
        if !app_id.is_finite()
            || app_id.fract() != 0.0
            || !(1.0..=u32::MAX as f64).contains(&app_id)
        {
            return Err(Error::from_reason(
                "App ID must be an integer from 1 to 4294967295",
            ));
        }
        if !matches!(
            kind.as_str(),
            "internet" | "lan" | "friends" | "favorites" | "history" | "spectator"
        ) {
            return Err(Error::from_reason("Unknown server list kind"));
        }
        let filters = filters.unwrap_or_default();
        if filters.len() > MAX_FILTERS || (kind == "lan" && !filters.is_empty()) {
            return Err(Error::from_reason(
                "At most 32 filters are allowed, and LAN does not accept filters",
            ));
        }
        let mut raw_filters = Vec::with_capacity(filters.len());
        for filter in filters {
            if filter.key.is_empty()
                || filter.key.len() > 255
                || filter.value.len() > 255
                || filter.key.contains('\0')
                || filter.value.contains('\0')
            {
                return Err(Error::from_reason("Filter keys must contain 1 to 255 bytes and values at most 255 bytes, without NUL"));
            }
            let mut raw = sys::MatchMakingKeyValuePair_t {
                m_szKey: [0; 256],
                m_szValue: [0; 256],
            };
            for (to, from) in raw.m_szKey.iter_mut().zip(filter.key.bytes()) {
                *to = from as _;
            }
            for (to, from) in raw.m_szValue.iter_mut().zip(filter.value.bytes()) {
                *to = from as _;
            }
            raw_filters.push(raw);
        }
        if !crate::client::is_initialized() {
            return Err(Error::from_reason("Steam client is not initialized"));
        }
        let _client = crate::client::get_client();
        runtime::without_dispatch(|| unsafe {
            let interface = sys::SteamAPI_SteamMatchmakingServers_v002();
            if interface.is_null() {
                return Err(Error::from_reason("Steam server browser unavailable"));
            }
            let callback = steamworksjs_server_list_response();
            let mut filter_pointers: Vec<_> = raw_filters
                .iter_mut()
                .map(|filter| filter as *mut sys::MatchMakingKeyValuePair_t)
                .collect();
            let filters_ptr = if filter_pointers.is_empty() {
                std::ptr::null_mut()
            } else {
                filter_pointers.as_mut_ptr()
            };
            let count = raw_filters.len() as u32;
            let app_id = app_id as u32;
            let handle = match kind.as_str() {
                "internet" => sys::SteamAPI_ISteamMatchmakingServers_RequestInternetServerList(
                    interface,
                    app_id,
                    filters_ptr,
                    count,
                    callback,
                ),
                "lan" => sys::SteamAPI_ISteamMatchmakingServers_RequestLANServerList(
                    interface, app_id, callback,
                ),
                "friends" => sys::SteamAPI_ISteamMatchmakingServers_RequestFriendsServerList(
                    interface,
                    app_id,
                    filters_ptr,
                    count,
                    callback,
                ),
                "favorites" => sys::SteamAPI_ISteamMatchmakingServers_RequestFavoritesServerList(
                    interface,
                    app_id,
                    filters_ptr,
                    count,
                    callback,
                ),
                "history" => sys::SteamAPI_ISteamMatchmakingServers_RequestHistoryServerList(
                    interface,
                    app_id,
                    filters_ptr,
                    count,
                    callback,
                ),
                "spectator" => sys::SteamAPI_ISteamMatchmakingServers_RequestSpectatorServerList(
                    interface,
                    app_id,
                    filters_ptr,
                    count,
                    callback,
                ),
                _ => unreachable!(),
            };
            if handle.is_null() {
                return Err(Error::from_reason(
                    "Steam could not create the server list request",
                ));
            }
            Ok(ServerList {
                interface: interface as usize,
                handle: Some(handle as usize),
            })
        })
    }
}
