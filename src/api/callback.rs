use napi_derive::napi;
use steamworks::sys;

struct TicketValidation {
    peer: u64,
    owner: u64,
    response: u32,
}

// The pinned safe callback panics for SDK response 10 (network identity failure).
unsafe impl steamworks::Callback for TicketValidation {
    const ID: i32 = sys::ValidateAuthTicketResponse_t_k_iCallback as i32;
    const SIZE: i32 = std::mem::size_of::<sys::ValidateAuthTicketResponse_t>() as i32;
    unsafe fn from_raw(raw: *mut std::ffi::c_void) -> Self {
        let event = raw.cast::<sys::ValidateAuthTicketResponse_t>();
        Self {
            peer: std::ptr::addr_of!((*event).m_SteamID.m_steamid.m_unAll64Bits).read_unaligned(),
            owner: std::ptr::addr_of!((*event).m_OwnerSteamID.m_steamid.m_unAll64Bits)
                .read_unaligned(),
            response: std::ptr::addr_of!((*event).m_eAuthSessionResponse)
                .cast::<u32>()
                .read_unaligned(),
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
    const SIZE: i32 = std::mem::size_of::<sys::GameRichPresenceJoinRequested_t>() as i32;
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
    const SIZE: i32 = std::mem::size_of::<sys::FriendRichPresenceUpdate_t>() as i32;
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

#[napi]
pub mod callback {
    use napi::{
        threadsafe_function::{ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode},
        JsFunction,
    };

    #[napi]
    pub struct Handle {
        handle: Option<steamworks::CallbackHandle>,
    }

    #[napi]
    impl Handle {
        #[napi]
        pub fn disconnect(&mut self) {
            if let Some(handle) = self.handle.take() {
                handle.disconnect();
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
    }

    #[napi(ts_generic_types = "C extends keyof import('./callbacks').CallbackReturns")]
    pub fn register(
        #[napi(ts_arg_type = "C")] steam_callback: SteamCallback,
        #[napi(ts_arg_type = "(value: import('./callbacks').CallbackReturns[C]) => void")] handler: JsFunction,
    ) -> napi::Result<Handle> {
        if let SteamCallback::ValidateAuthTicketResponse = steam_callback {
            let listener: ThreadsafeFunction<super::TicketValidation, ErrorStrategy::Fatal> =
                handler.create_threadsafe_function(
                    0,
                    |ctx: napi::threadsafe_function::ThreadSafeCallContext<
                        super::TicketValidation,
                    >| {
                        let mut value = ctx.env.create_object()?;
                        value.set_named_property(
                            "steam_id",
                            ctx.env.create_bigint_from_u64(ctx.value.peer)?,
                        )?;
                        value.set_named_property(
                            "owner_steam_id",
                            ctx.env.create_bigint_from_u64(ctx.value.owner)?,
                        )?;
                        value.set_named_property("response", ctx.value.response)?;
                        Ok(vec![value])
                    },
                )?;
            let handle = crate::client::get_client().register_callback(move |event| {
                listener.call(event, ThreadsafeFunctionCallMode::Blocking);
            });
            return Ok(Handle {
                handle: Some(handle),
            });
        }
        let threadsafe_handler: ThreadsafeFunction<serde_json::Value, ErrorStrategy::Fatal> =
            handler.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;

        let handle = match steam_callback {
            SteamCallback::ValidateAuthTicketResponse => unreachable!(),
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
                register_callback::<steamworks::PersonaStateChange>(threadsafe_handler)
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
                register_callback::<steamworks::GameOverlayActivated>(threadsafe_handler)
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
            SteamCallback::ScreenshotRequested => crate::client::get_client().register_callback(
                move |_: steamworks::screenshots::ScreenshotRequested| {
                    threadsafe_handler
                        .call(serde_json::json!({}), ThreadsafeFunctionCallMode::Blocking);
                },
            ),
            SteamCallback::ScreenshotReady => crate::client::get_client().register_callback(
                move |event: steamworks::screenshots::ScreenshotReady| {
                    let value = match event.local_handle {
                        Ok(handle) => serde_json::json!({"handle": handle, "error": null}),
                        Err(error) => {
                            serde_json::json!({"handle": null, "error": error.to_string()})
                        }
                    };
                    threadsafe_handler.call(value, ThreadsafeFunctionCallMode::Blocking);
                },
            ),
        };

        Ok(Handle {
            handle: Some(handle),
        })
    }

    fn register_callback<C>(
        threadsafe_handler: ThreadsafeFunction<serde_json::Value, ErrorStrategy::Fatal>,
    ) -> steamworks::CallbackHandle
    where
        C: steamworks::Callback + serde::Serialize,
    {
        let client = crate::client::get_client();
        client.register_callback(move |value: C| {
            let value = serde_json::to_value(&value).unwrap();
            threadsafe_handler.call(value, ThreadsafeFunctionCallMode::Blocking);
        })
    }
}
