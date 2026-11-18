use napi_derive::napi;

#[napi]
pub mod networking_messages {
    use crate::api::localplayer::PlayerSteamId;
    use napi::{
        bindgen_prelude::{BigInt, Buffer},
        threadsafe_function::{ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode},
        Env, Error, JsFunction, Result, Status,
    };
    use steamworks::{
        networking_types::{NetworkingIdentity, SendFlags},
        sys, Callback, SteamId,
    };

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
        // CSteamID individual account: universe, type, desktop instance, account number.
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

    /// Queues a message and implicitly accepts a session from this peer. Success is not delivery.
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

    /// Polls one channel (default 0) for up to 1–32 messages (default 10).
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

    // Steam's manual dispatcher lends the payload only for this callback. Copy the
    // identity, not steamworks-rs's SessionRequest guard (its Drop closes the peer).
    unsafe impl Callback for SessionRequest {
        const ID: i32 = sys::SteamNetworkingMessagesSessionRequest_t_k_iCallback as i32;
        const SIZE: i32 =
            std::mem::size_of::<sys::SteamNetworkingMessagesSessionRequest_t>() as i32;

        unsafe fn from_raw(raw: *mut std::ffi::c_void) -> Self {
            let remote =
                (*raw.cast::<sys::SteamNetworkingMessagesSessionRequest_t>()).m_identityRemote;
            Self {
                steam_id: NetworkingIdentity::from(remote)
                    .steam_id()
                    .map(PlayerSteamId::from_steamid),
            }
        }
    }

    /// Notifies without accepting. Omit the handler to stop listening.
    #[napi(strict)]
    pub fn set_session_request_handler(
        env: Env,
        #[napi(ts_arg_type = "(request: SessionRequest) => void")] handler: Option<JsFunction>,
    ) -> Result<()> {
        let client = crate::client::get_client();
        if let Some(handler) = handler {
            let mut callback: ThreadsafeFunction<SessionRequest, ErrorStrategy::Fatal> =
                handler.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;
            callback.unref(&env)?;
            client.register_callback(move |request: SessionRequest| {
                callback.call(request, ThreadsafeFunctionCallMode::NonBlocking);
            });
        } else {
            client.register_callback(|_: SessionRequest| {});
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
            // The SDK defines a zeroed identity as Invalid; SetSteamID64 sets its type and size.
            let mut identity: sys::SteamNetworkingIdentity = std::mem::zeroed();
            sys::SteamAPI_SteamNetworkingIdentity_SetSteamID64(&mut identity, peer.raw());
            Ok(operation(messages, &identity))
        }
    }

    /// Accept a known peer's pending request without sending a message.
    #[napi(strict)]
    pub fn accept_session_with_user(steam_id64: BigInt) -> Result<bool> {
        update_session(steam_id64, |messages, id| unsafe {
            sys::SteamAPI_ISteamNetworkingMessages_AcceptSessionWithUser(messages, id)
        })
    }

    /// Close the peer's session on all channels. This does not block future requests.
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

    /// Replaces the failure handler. Omit the handler to stop listening.
    #[napi]
    pub fn set_session_failed_handler(
        env: Env,
        #[napi(ts_arg_type = "(failure: SessionFailure) => void")] handler: Option<JsFunction>,
    ) -> Result<()> {
        let messages = crate::client::get_client().networking_messages();
        if let Some(handler) = handler {
            let mut callback: ThreadsafeFunction<SessionFailure, ErrorStrategy::Fatal> =
                handler.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;
            callback.unref(&env)?;
            messages.session_failed_callback(move |info| {
                let steam_id = info.identity_remote().and_then(|id| id.steam_id());
                callback.call(
                    SessionFailure {
                        steam_id: steam_id.map(PlayerSteamId::from_steamid),
                    },
                    ThreadsafeFunctionCallMode::NonBlocking,
                );
            });
        } else {
            messages.session_failed_callback(|_| {});
        }
        Ok(())
    }
}
