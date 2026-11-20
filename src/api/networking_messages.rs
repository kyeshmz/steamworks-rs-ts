use napi_derive::napi;

#[napi]
pub mod networking_messages {
    use crate::api::localplayer::PlayerSteamId;
    use napi::{
        bindgen_prelude::{BigInt, Buffer, Function},
        threadsafe_function::ThreadsafeFunctionCallMode,
        Error, Result, Status,
    };
    use std::sync::Mutex;
    use steamworks::{
        networking_types::{NetworkingIdentity, SendFlags},
        sys, Callback, CallbackHandle, SteamId,
    };

    static REQUEST_HANDLER: Mutex<Option<CallbackHandle>> = Mutex::new(None);
    static FAILURE_HANDLER: Mutex<Option<CallbackHandle>> = Mutex::new(None);

    const MAX_SEND_BYTES: usize = 512 * 1024;
    const MAX_BATCH: u32 = 32;

    #[napi(object)]
    pub struct Message {
        pub data: Buffer,
        pub steam_id: Option<PlayerSteamId>,
    }

    #[napi]
    pub enum MessageSendType {
        ReliableNoNagle,
        Reliable,
        Unreliable,
        UnreliableNoDelay,
    }

    fn integer(name: &str, value: f64, min: u32, max: u32) -> Result<u32> {
        if value.fract() == 0.0 && (min as f64..=max as f64).contains(&value) {
            Ok(value as u32)
        } else {
            Err(Error::new(
                Status::InvalidArg,
                format!("{name} must be an integer from {min} to {max}"),
            ))
        }
    }

    fn checked_channel(value: Option<f64>) -> Result<u32> {
        integer("channel", value.unwrap_or(0.0), 0, i32::MAX as u32)
    }

    fn peer(id: BigInt) -> Result<SteamId> {
        let (negative, raw, lossless) = id.get_u64();
        if !negative
            && lossless
            && (1..=4).contains(&(raw >> 56))
            && (raw >> 52) & 0xf == 1
            && (raw >> 32) & 0xfffff == 1
            && raw as u32 != 0
        {
            Ok(SteamId::from_raw(raw))
        } else {
            Err(Error::new(Status::InvalidArg, "Expected a user SteamID64"))
        }
    }

    #[napi(strict)]
    pub fn send_message_to_user(
        steam_id64: BigInt,
        data: Buffer,
        channel: Option<f64>,
        send_type: Option<MessageSendType>,
    ) -> Result<()> {
        let peer = peer(steam_id64)?;
        let channel = checked_channel(channel)?;
        if data.len() > MAX_SEND_BYTES {
            return Err(Error::new(Status::InvalidArg, "Message exceeds 512 KiB"));
        }
        let flags = match send_type.unwrap_or(MessageSendType::ReliableNoNagle) {
            MessageSendType::ReliableNoNagle => SendFlags::RELIABLE_NO_NAGLE,
            MessageSendType::Reliable => SendFlags::RELIABLE,
            MessageSendType::Unreliable => SendFlags::UNRELIABLE,
            MessageSendType::UnreliableNoDelay => SendFlags::UNRELIABLE_NO_DELAY,
        };
        crate::client::get_client()
            .networking_messages()
            .send_message_to_user(
                NetworkingIdentity::new_steam_id(peer),
                flags | SendFlags::AUTO_RESTART_BROKEN_SESSION,
                &data,
                channel,
            )
            .map_err(|err| Error::from_reason(format!("Failed to send Steam message: {err}")))
    }

    #[napi(strict)]
    pub fn receive_messages_on_channel(
        channel: Option<f64>,
        max_messages: Option<f64>,
    ) -> Result<Vec<Message>> {
        let channel = checked_channel(channel)?;
        let count = integer("maxMessages", max_messages.unwrap_or(10.0), 1, MAX_BATCH)?;
        Ok(crate::client::get_client()
            .networking_messages()
            .receive_messages_on_channel(channel, count as usize)
            .into_iter()
            .map(|message| Message {
                data: message.data().to_vec().into(),
                steam_id: message
                    .identity_peer()
                    .steam_id()
                    .map(PlayerSteamId::from_steamid),
            })
            .collect())
    }

    #[napi(object)]
    pub struct SessionFailure {
        pub steam_id: Option<PlayerSteamId>,
    }

    #[napi(object)]
    pub struct SessionRequest {
        pub steam_id: Option<PlayerSteamId>,
    }

    unsafe impl Callback for SessionRequest {
        const ID: i32 = sys::SteamNetworkingMessagesSessionRequest_t_k_iCallback as i32;

        unsafe fn from_raw(raw: *mut std::ffi::c_void) -> Self {
            let remote = raw
                .cast::<sys::SteamNetworkingMessagesSessionRequest_t>()
                .read_unaligned()
                .m_identityRemote;
            Self {
                steam_id: NetworkingIdentity::from(remote)
                    .steam_id()
                    .map(PlayerSteamId::from_steamid),
            }
        }
    }

    #[napi(strict)]
    pub fn set_session_request_handler(
        #[napi(ts_arg_type = "(request: SessionRequest) => void")] handler: Option<
            Function<'_, SessionRequest, ()>,
        >,
    ) -> Result<()> {
        let callback = handler
            .map(|handler| handler.build_threadsafe_function().weak::<true>().build())
            .transpose()?;
        let mut slot = REQUEST_HANDLER
            .lock()
            .map_err(|_| Error::from_reason("Session handler lock poisoned"))?;
        drop(slot.take());
        if let Some(callback) = callback {
            *slot = Some(crate::client::get_client().register_callback(
                move |request: SessionRequest| {
                    callback.call(request, ThreadsafeFunctionCallMode::NonBlocking);
                },
            ));
        }
        Ok(())
    }

    fn update_session(
        steam_id64: BigInt,
        operation: impl FnOnce(
            *mut sys::ISteamNetworkingMessages,
            *const sys::SteamNetworkingIdentity,
        ) -> bool,
    ) -> Result<bool> {
        let peer = peer(steam_id64)?;
        let _client = crate::client::get_client();
        unsafe {
            let messages = sys::SteamAPI_SteamNetworkingMessages_SteamAPI_v002();
            if messages.is_null() {
                return Err(Error::from_reason("Steam Networking Messages unavailable"));
            }
            let mut identity: sys::SteamNetworkingIdentity = std::mem::zeroed();
            sys::SteamAPI_SteamNetworkingIdentity_SetSteamID64(&mut identity, peer.raw());
            Ok(operation(messages, &identity))
        }
    }

    #[napi(strict)]
    pub fn accept_session_with_user(steam_id64: BigInt) -> Result<bool> {
        update_session(steam_id64, |messages, id| unsafe {
            sys::SteamAPI_ISteamNetworkingMessages_AcceptSessionWithUser(messages, id)
        })
    }

    #[napi(strict)]
    pub fn close_session_with_user(steam_id64: BigInt) -> Result<bool> {
        update_session(steam_id64, |messages, id| unsafe {
            sys::SteamAPI_ISteamNetworkingMessages_CloseSessionWithUser(messages, id)
        })
    }

    #[napi(strict)]
    pub fn close_channel_with_user(steam_id64: BigInt, channel: f64) -> Result<bool> {
        let channel = checked_channel(Some(channel))? as i32;
        update_session(steam_id64, |messages, id| unsafe {
            sys::SteamAPI_ISteamNetworkingMessages_CloseChannelWithUser(messages, id, channel)
        })
    }

    #[napi(object)]
    pub struct SessionConnectionInfo {
        pub state: i32,
        pub steam_id64: Option<BigInt>,
        pub listen_socket: u32,
        pub end_reason: i32,
        pub ping: i32,
        pub quality_local: f64,
        pub quality_remote: f64,
    }

    #[napi(strict)]
    pub fn get_session_connection_info(
        steam_id64: BigInt,
    ) -> Result<Option<SessionConnectionInfo>> {
        let peer = peer(steam_id64)?;
        let _client = crate::client::get_client();
        unsafe {
            let messages = sys::SteamAPI_SteamNetworkingMessages_SteamAPI_v002();
            if messages.is_null() {
                return Err(Error::from_reason("Steam Networking Messages unavailable"));
            }
            let mut identity = std::mem::zeroed();
            sys::SteamAPI_SteamNetworkingIdentity_SetSteamID64(&mut identity, peer.raw());
            let mut info: sys::SteamNetConnectionInfo_t = std::mem::zeroed();
            let mut status: sys::SteamNetConnectionRealTimeStatus_t = std::mem::zeroed();
            let state = sys::SteamAPI_ISteamNetworkingMessages_GetSessionConnectionInfo(
                messages,
                &identity,
                &mut info,
                &mut status,
            ) as i32;
            if state == 0 {
                return Ok(None);
            }
            Ok(Some(SessionConnectionInfo {
                state,
                steam_id64: NetworkingIdentity::from(info.m_identityRemote)
                    .steam_id()
                    .map(|id| id.raw().into()),
                listen_socket: info.m_hListenSocket,
                end_reason: info.m_eEndReason,
                ping: status.m_nPing,
                quality_local: status.m_flConnectionQualityLocal as f64,
                quality_remote: status.m_flConnectionQualityRemote as f64,
            }))
        }
    }

    unsafe impl Callback for SessionFailure {
        const ID: i32 = sys::SteamNetworkingMessagesSessionFailed_t_k_iCallback as i32;

        unsafe fn from_raw(raw: *mut std::ffi::c_void) -> Self {
            let event = raw
                .cast::<sys::SteamNetworkingMessagesSessionFailed_t>()
                .read_unaligned();
            Self {
                steam_id: NetworkingIdentity::from(event.m_info.m_identityRemote)
                    .steam_id()
                    .map(PlayerSteamId::from_steamid),
            }
        }
    }

    #[napi]
    pub fn set_session_failed_handler(
        #[napi(ts_arg_type = "(failure: SessionFailure) => void")] handler: Option<
            Function<'_, SessionFailure, ()>,
        >,
    ) -> Result<()> {
        let callback = handler
            .map(|handler| handler.build_threadsafe_function().weak::<true>().build())
            .transpose()?;
        let mut slot = FAILURE_HANDLER
            .lock()
            .map_err(|_| Error::from_reason("Session handler lock poisoned"))?;
        drop(slot.take());
        if let Some(callback) = callback {
            *slot = Some(crate::client::get_client().register_callback(
                move |failure: SessionFailure| {
                    callback.call(failure, ThreadsafeFunctionCallMode::NonBlocking);
                },
            ));
        }
        Ok(())
    }
}
