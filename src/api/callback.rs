use napi_derive::napi;
use steamworks::sys;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct OverlayActivation {
    active: bool,
    user_initiated: bool,
    app_id: u32,
}

unsafe impl steamworks::Callback for OverlayActivation {
    const ID: i32 = sys::GameOverlayActivated_t_k_iCallback as i32;

    unsafe fn from_raw(raw: *mut std::ffi::c_void) -> Self {
        let event = raw.cast::<sys::GameOverlayActivated_t>().read_unaligned();
        Self {
            active: event.m_bActive != 0,
            user_initiated: event.m_bUserInitiated,
            app_id: event.m_nAppID,
        }
    }
}

#[derive(serde::Serialize)]
struct PresenceJoin {
    friend_steam_id: u64,
    connect: String,
}

unsafe impl steamworks::Callback for PresenceJoin {
    const ID: i32 = sys::GameRichPresenceJoinRequested_t_k_iCallback as i32;

    unsafe fn from_raw(raw: *mut std::ffi::c_void) -> Self {
        let event = raw
            .cast::<sys::GameRichPresenceJoinRequested_t>()
            .read_unaligned();
        let bytes: Vec<u8> = event
            .m_rgchConnect
            .into_iter()
            .take_while(|&b| b != 0)
            .map(|b| b as u8)
            .collect();
        Self {
            friend_steam_id: event.m_steamIDFriend.m_steamid.m_unAll64Bits,
            connect: String::from_utf8_lossy(&bytes).into_owned(),
        }
    }
}

#[derive(serde::Serialize)]
struct PresenceUpdate {
    friend_steam_id: u64,
    app_id: u32,
}

unsafe impl steamworks::Callback for PresenceUpdate {
    const ID: i32 = sys::FriendRichPresenceUpdate_t_k_iCallback as i32;

    unsafe fn from_raw(raw: *mut std::ffi::c_void) -> Self {
        let event = raw
            .cast::<sys::FriendRichPresenceUpdate_t>()
            .read_unaligned();
        Self {
            friend_steam_id: event.m_steamIDFriend.m_steamid.m_unAll64Bits,
            app_id: event.m_nAppID,
        }
    }
}

mod events {
    #[napi_derive::napi(object, object_from_js = false)]
    pub struct LobbyCreated {
        pub result: u32,
        pub lobby: u64,
    }

    #[napi_derive::napi(object, object_from_js = false)]
    pub struct LobbyEnter {
        pub lobby: u64,
        #[napi(js_name = "chat_permissions")]
        pub chat_permissions: u32,
        pub blocked: bool,
        #[napi(js_name = "chat_room_enter_response")]
        pub chat_room_enter_response: String,
    }

    #[napi_derive::napi(object, object_from_js = false)]
    pub struct StatsReceived {
        #[napi(js_name = "steam_id")]
        pub steam_id: u64,
        #[napi(js_name = "game_id")]
        pub game_id: u64,
        pub error: serde_json::Value,
    }

    #[napi_derive::napi(object, object_from_js = false)]
    pub struct StatsStored {
        #[napi(js_name = "game_id")]
        pub game_id: u64,
        pub error: serde_json::Value,
    }

    #[napi_derive::napi(object, object_from_js = false)]
    pub struct AchievementStored {
        #[napi(js_name = "game_id")]
        pub game_id: u64,
        #[napi(js_name = "achievement_name")]
        pub achievement_name: String,
        #[napi(js_name = "current_progress")]
        pub current_progress: u32,
        #[napi(js_name = "max_progress")]
        pub max_progress: u32,
    }

    #[napi_derive::napi(object, object_from_js = false)]
    pub struct AchievementIconFetched {
        #[napi(js_name = "game_id")]
        pub game_id: u64,
        #[napi(js_name = "achievement_name")]
        pub achievement_name: String,
        pub achieved: bool,
        #[napi(js_name = "icon_handle")]
        pub icon_handle: i32,
    }

    #[napi_derive::napi(object, object_from_js = false)]
    pub struct DownloadItemResult {
        #[napi(js_name = "app_id")]
        pub app_id: u32,
        #[napi(js_name = "published_file_id")]
        pub published_file_id: u64,
        pub error: serde_json::Value,
    }

    #[napi_derive::napi(object, object_from_js = false)]
    pub struct RemotePlay {
        pub session: u32,
    }

    #[napi_derive::napi(object, object_from_js = false)]
    pub struct GlobalStatsReceived {
        #[napi(js_name = "game_id")]
        pub game_id: u64,
        pub error: serde_json::Value,
    }

    #[napi_derive::napi(object, object_from_js = false)]
    pub struct DeleteItemResult {
        #[napi(js_name = "published_file_id")]
        pub published_file_id: u64,
        pub error: serde_json::Value,
    }
}

#[napi]
pub mod callback {
    use crate::api::steam_events::TicketValidation;
    use napi::{
        bindgen_prelude::{Function, JsValuesTupleIntoVec},
        threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode},
    };
    use std::{
        any::Any,
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    type Listener<T> = ThreadsafeFunction<T, (), T, napi::Status, false>;
    type Slot<T> = Arc<Mutex<Option<(u64, Listener<T>)>>>;

    #[derive(Default)]
    struct Registry {
        slots: HashMap<i32, Box<dyn Any + Send>>,
        handles: Vec<steamworks::CallbackHandle>,
        next: u64,
    }

    lazy_static! {
        static ref REGISTRY: Mutex<Registry> = Mutex::new(Registry::default());
    }

    #[napi]
    pub struct Handle {
        disconnect: Option<Box<dyn FnOnce() + Send>>,
    }

    #[napi]
    impl Handle {
        #[napi]
        pub fn disconnect(&mut self) {
            if let Some(disconnect) = self.disconnect.take() {
                disconnect();
            }
        }
    }

    #[napi]
    pub enum SteamCallback {
        PersonaStateChange,
        SteamServersConnected,
        SteamServersDisconnected,
        SteamServerConnectFailure,
        LobbyDataUpdate,
        LobbyChatUpdate,
        P2PSessionRequest,
        P2PSessionConnectFail,
        GameLobbyJoinRequested,
        MicroTxnAuthorizationResponse,
        GameOverlayActivated,
        LobbyChatMsg,
        GameRichPresenceJoinRequested,
        FriendRichPresenceUpdate,
        ScreenshotRequested,
        ScreenshotReady,
        ValidateAuthTicketResponse,
        SteamInputDeviceConnected,
        SteamInputDeviceDisconnected,
        SteamInventoryDefinitionUpdate,
        SteamInventoryFullUpdate,
        LobbyCreated,
        LobbyEnter,
        NewUrlLaunchParameters,
        UserStatsReceived,
        UserStatsStored,
        UserAchievementStored,
        UserAchievementIconFetched,
        DownloadItemResult,
        RemotePlayConnected,
        RemotePlayDisconnected,
        RelayNetworkStatus,
        GlobalStatsReceived,
        DeleteItemResult,
    }

    #[napi(ts_generic_types = "C extends keyof import('./callbacks').RuntimeCallbackReturns")]
    pub fn register_typed(
        #[napi(ts_arg_type = "C")] steam_callback: SteamCallback,
        #[napi(ts_arg_type = "(value: import('./callbacks').RuntimeCallbackReturns[C]) => void")]
        handler: Function<'_, serde_json::Value, ()>,
    ) -> napi::Result<Handle> {
        register(steam_callback, handler)
    }

    #[napi(ts_generic_types = "C extends keyof import('./callbacks').CallbackReturns")]
    pub fn register(
        #[napi(ts_arg_type = "C")] steam_callback: SteamCallback,
        #[napi(ts_arg_type = "(value: import('./callbacks').CallbackReturns[C]) => void")] handler: Function<'_, serde_json::Value, ()>,
    ) -> napi::Result<Handle> {
        if let SteamCallback::ValidateAuthTicketResponse = steam_callback {
            let listener = handler
                .build_threadsafe_function::<TicketValidation>()
                .build_callback(|ctx| Ok(ctx.value))?;
            return Ok(register_mapped::<TicketValidation, TicketValidation>(
                listener,
                |event| event,
            ));
        }
        macro_rules! typed {
            ($source:ty, $target:ty, $convert:expr) => {{
                let listener = handler
                    .build_threadsafe_function::<$target>()
                    .build_callback(|ctx| Ok(ctx.value))?;
                return Ok(register_mapped::<$source, $target>(listener, $convert));
            }};
        }
        match steam_callback {
            SteamCallback::LobbyCreated => {
                typed!(steamworks::LobbyCreated, super::events::LobbyCreated, |e| {
                    super::events::LobbyCreated {
                        result: e.result,
                        lobby: e.lobby.raw(),
                    }
                })
            }
            SteamCallback::LobbyEnter => {
                typed!(steamworks::LobbyEnter, super::events::LobbyEnter, |e| {
                    super::events::LobbyEnter {
                        lobby: e.lobby.raw(),
                        chat_permissions: e.chat_permissions,
                        blocked: e.blocked,
                        chat_room_enter_response: format!("{:?}", e.chat_room_enter_response),
                    }
                })
            }
            SteamCallback::NewUrlLaunchParameters => {
                let listener = handler.build_threadsafe_function().build()?;
                return Ok(register_mapped::<steamworks::NewUrlLaunchParameters, _>(
                    listener,
                    |_| serde_json::json!({}),
                ));
            }
            SteamCallback::UserStatsReceived => typed!(
                steamworks::UserStatsReceived,
                super::events::StatsReceived,
                |e| super::events::StatsReceived {
                    steam_id: e.steam_id.raw(),
                    game_id: e.game_id.raw(),
                    error: serde_json::json!(e.result.err().map(|e| e.to_string()))
                }
            ),
            SteamCallback::UserStatsStored => typed!(
                steamworks::UserStatsStored,
                super::events::StatsStored,
                |e| super::events::StatsStored {
                    game_id: e.game_id.raw(),
                    error: serde_json::json!(e.result.err().map(|e| e.to_string()))
                }
            ),
            SteamCallback::UserAchievementStored => typed!(
                steamworks::UserAchievementStored,
                super::events::AchievementStored,
                |e| super::events::AchievementStored {
                    game_id: e.game_id.raw(),
                    achievement_name: e.achievement_name,
                    current_progress: e.current_progress,
                    max_progress: e.max_progress
                }
            ),
            SteamCallback::UserAchievementIconFetched => typed!(
                steamworks::UserAchievementIconFetched,
                super::events::AchievementIconFetched,
                |e| super::events::AchievementIconFetched {
                    game_id: e.game_id.raw(),
                    achievement_name: e.achievement_name,
                    achieved: e.achieved,
                    icon_handle: e.icon_handle
                }
            ),
            SteamCallback::DownloadItemResult => typed!(
                steamworks::DownloadItemResult,
                super::events::DownloadItemResult,
                |e| super::events::DownloadItemResult {
                    app_id: e.app_id.0,
                    published_file_id: e.published_file_id.0,
                    error: serde_json::json!(e.error.map(|e| e.to_string()))
                }
            ),
            SteamCallback::RemotePlayConnected => typed!(
                steamworks::RemotePlayConnected,
                super::events::RemotePlay,
                |e| super::events::RemotePlay {
                    session: e.session.raw()
                }
            ),
            SteamCallback::RemotePlayDisconnected => typed!(
                steamworks::RemotePlayDisconnected,
                super::events::RemotePlay,
                |e| super::events::RemotePlay {
                    session: e.session.raw()
                }
            ),
            SteamCallback::RelayNetworkStatus => typed!(
                crate::api::steam_events::RelayNetworkStatus,
                crate::api::steam_events::RelayNetworkStatus,
                |e| e
            ),
            SteamCallback::GlobalStatsReceived => typed!(
                steamworks::GlobalStatsReceived,
                super::events::GlobalStatsReceived,
                |e| super::events::GlobalStatsReceived {
                    game_id: e.game_id.raw(),
                    error: serde_json::json!(e.result.err().map(|e| e.to_string()))
                }
            ),
            SteamCallback::DeleteItemResult => typed!(
                steamworks::DeleteItemResult,
                super::events::DeleteItemResult,
                |e| super::events::DeleteItemResult {
                    published_file_id: e.published_file_id.0,
                    error: serde_json::json!(e.error.map(|e| e.to_string()))
                }
            ),
            _ => {}
        }
        let threadsafe_handler = handler.build_threadsafe_function().build()?;

        let handle = match steam_callback {
            SteamCallback::ValidateAuthTicketResponse => unreachable!(),
            SteamCallback::LobbyCreated
            | SteamCallback::LobbyEnter
            | SteamCallback::NewUrlLaunchParameters
            | SteamCallback::UserStatsReceived
            | SteamCallback::UserStatsStored
            | SteamCallback::UserAchievementStored
            | SteamCallback::UserAchievementIconFetched
            | SteamCallback::DownloadItemResult
            | SteamCallback::RemotePlayConnected
            | SteamCallback::RemotePlayDisconnected
            | SteamCallback::RelayNetworkStatus
            | SteamCallback::GlobalStatsReceived
            | SteamCallback::DeleteItemResult => unreachable!(),
            SteamCallback::SteamInputDeviceConnected => {
                register_callback::<crate::api::steam_events::DeviceConnected>(threadsafe_handler)
            }
            SteamCallback::SteamInputDeviceDisconnected => register_callback::<
                crate::api::steam_events::DeviceDisconnected,
            >(threadsafe_handler),
            SteamCallback::SteamInventoryDefinitionUpdate => register_callback::<
                crate::api::steam_events::InventoryDefinitionUpdate,
            >(threadsafe_handler),
            SteamCallback::SteamInventoryFullUpdate => register_callback::<
                crate::api::steam_events::InventoryFullUpdate,
            >(threadsafe_handler),
            SteamCallback::PersonaStateChange => {
                register_mapped::<steamworks::PersonaStateChange, _>(threadsafe_handler, |event| {
                    serde_json::json!({
                        "steam_id": event.steam_id.raw(),
                        "flags": { "bits": event.flags.bits() }
                    })
                })
            }
            SteamCallback::SteamServersConnected => {
                register_callback::<steamworks::SteamServersConnected>(threadsafe_handler)
            }
            SteamCallback::SteamServersDisconnected => {
                register_callback::<steamworks::SteamServersDisconnected>(threadsafe_handler)
            }
            SteamCallback::SteamServerConnectFailure => {
                register_callback::<steamworks::SteamServerConnectFailure>(threadsafe_handler)
            }
            SteamCallback::LobbyDataUpdate => {
                register_callback::<steamworks::LobbyDataUpdate>(threadsafe_handler)
            }
            SteamCallback::LobbyChatUpdate => {
                register_callback::<steamworks::LobbyChatUpdate>(threadsafe_handler)
            }
            SteamCallback::P2PSessionRequest => {
                register_callback::<steamworks::P2PSessionRequest>(threadsafe_handler)
            }
            SteamCallback::P2PSessionConnectFail => {
                register_callback::<steamworks::P2PSessionConnectFail>(threadsafe_handler)
            }
            SteamCallback::GameLobbyJoinRequested => {
                register_callback::<steamworks::GameLobbyJoinRequested>(threadsafe_handler)
            }
            SteamCallback::MicroTxnAuthorizationResponse => {
                register_callback::<steamworks::MicroTxnAuthorizationResponse>(threadsafe_handler)
            }
            SteamCallback::GameOverlayActivated => {
                register_callback::<super::OverlayActivation>(threadsafe_handler)
            }
            SteamCallback::LobbyChatMsg => {
                register_callback::<steamworks::LobbyChatMsg>(threadsafe_handler)
            }
            SteamCallback::GameRichPresenceJoinRequested => {
                register_callback::<super::PresenceJoin>(threadsafe_handler)
            }
            SteamCallback::FriendRichPresenceUpdate => {
                register_callback::<super::PresenceUpdate>(threadsafe_handler)
            }
            SteamCallback::ScreenshotRequested => {
                register_mapped::<steamworks::screenshots::ScreenshotRequested, _>(
                    threadsafe_handler,
                    |_| serde_json::json!({}),
                )
            }
            SteamCallback::ScreenshotReady => {
                register_mapped::<steamworks::screenshots::ScreenshotReady, _>(
                    threadsafe_handler,
                    |event| match event.local_handle {
                        Ok(handle) => serde_json::json!({"handle": handle, "error": null}),
                        Err(error) => {
                            serde_json::json!({"handle": null, "error": error.to_string()})
                        }
                    },
                )
            }
        };

        Ok(handle)
    }

    fn register_callback<C>(
        threadsafe_handler: ThreadsafeFunction<
            serde_json::Value,
            (),
            serde_json::Value,
            napi::Status,
            false,
        >,
    ) -> Handle
    where
        C: steamworks::Callback + serde::Serialize + 'static,
    {
        register_mapped(threadsafe_handler, |value: C| {
            serde_json::to_value(&value).unwrap()
        })
    }

    fn register_mapped<C, T>(listener: Listener<T>, convert: fn(C) -> T) -> Handle
    where
        C: steamworks::Callback + 'static,
        T: JsValuesTupleIntoVec + Send + 'static,
    {
        let mut registry = REGISTRY.lock().unwrap();
        let slot: Slot<T> = if let Some(slot) = registry.slots.get(&C::ID) {
            slot.downcast_ref::<Slot<T>>().unwrap().clone()
        } else {
            let slot: Slot<T> = Arc::new(Mutex::new(None));
            let destination = Arc::clone(&slot);
            let handle = crate::client::get_client().register_callback(move |event: C| {
                if let Some((_, listener)) = &*destination.lock().unwrap() {
                    listener.call(convert(event), ThreadsafeFunctionCallMode::NonBlocking);
                }
            });
            registry.slots.insert(C::ID, Box::new(Arc::clone(&slot)));
            registry.handles.push(handle);
            slot
        };
        registry.next = registry
            .next
            .checked_add(1)
            .expect("callback generation exhausted");
        let generation = registry.next;
        *slot.lock().unwrap() = Some((generation, listener));
        Handle {
            disconnect: Some(Box::new(move || {
                let mut current = slot.lock().unwrap();
                if current.as_ref().is_some_and(|(id, _)| *id == generation) {
                    current.take();
                }
            })),
        }
    }
}
