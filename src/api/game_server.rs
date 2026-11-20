use napi::{Error, Result};
use napi_derive::napi;
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use steamworks::{CallbackHandle, Server};

struct Hosted {
    socket: u32,
    connections: HashMap<u64, u32>,
    events: VecDeque<game_server::HostedEvent>,
    dropped: u32,
}

struct Dedicated {
    sdk: Server,
    phase: Arc<Mutex<game_server::ConnectionState>>,
    callbacks: Option<[CallbackHandle; 3]>,
    auth_listener: Option<(u64, CallbackHandle)>,
    server_listener: Option<(u64, Vec<CallbackHandle>)>,
    message_listener: Option<(u64, CallbackHandle)>,
    p2p_listener: Option<(u64, CallbackHandle)>,
    shared_socket: bool,
    hosted: Option<Arc<Mutex<Hosted>>>,
    hosted_callback: Option<CallbackHandle>,
}

impl Drop for Dedicated {
    fn drop(&mut self) {
        self.close_hosted();
        self.auth_listener.take();
        self.server_listener.take();
        self.message_listener.take();
        self.p2p_listener.take();
        self.callbacks.take();
    }
}

impl Dedicated {
    fn close_hosted(&mut self) -> bool {
        self.hosted_callback.take();
        let Some(hosted) = self.hosted.take() else {
            return false;
        };
        let hosted = hosted.lock().unwrap();
        let sockets =
            unsafe { steamworks::sys::SteamAPI_SteamGameServerNetworkingSockets_SteamAPI_v012() };
        if sockets.is_null() {
            return false;
        }
        unsafe {
            for connection in hosted.connections.values() {
                steamworks::sys::SteamAPI_ISteamNetworkingSockets_CloseConnection(
                    sockets,
                    *connection,
                    0,
                    std::ptr::null(),
                    false,
                );
            }
            steamworks::sys::SteamAPI_ISteamNetworkingSockets_CloseListenSocket(
                sockets,
                hosted.socket,
            )
        }
    }
}

static SERVER: Mutex<Option<Dedicated>> = Mutex::new(None);
static LISTENER_ID: AtomicU64 = AtomicU64::new(1);

#[napi]
pub mod game_server {
    use super::*;
    use crate::api::steam_events::TicketValidation;
    use napi::{
        bindgen_prelude::{BigInt, Buffer, Function},
        threadsafe_function::ThreadsafeFunctionCallMode,
        Status,
    };
    use std::net::{Ipv4Addr, SocketAddrV4};
    use steamworks::{
        GSClientApprove, GSClientDeny, GSClientGroupStatus, GSClientKick, ServerMode, SteamId,
        SteamServerConnectFailure, SteamServersConnected, SteamServersDisconnected,
    };

    #[napi(string_enum)]
    pub enum Mode {
        NoAuthentication,
        Authentication,
        AuthenticationAndSecure,
    }

    #[napi(string_enum)]
    #[derive(Clone, Copy)]
    pub enum ConnectionState {
        Ready,
        Connecting,
        Connected,
        Failed,
        Disconnected,
    }

    #[napi(object)]
    pub struct ServerListing {
        pub server_name: String,
        pub map_name: String,
        pub max_players: f64,
        pub bot_players: Option<f64>,
        pub password_protected: Option<bool>,
    }

    #[napi(object)]
    pub struct ServerOptions {
        pub bind_ip: String,
        pub game_port: f64,
        pub query_port: f64,
        pub mode: Mode,
        pub version: String,
        pub product: String,
        pub description: String,
        pub listing: ServerListing,
    }

    #[napi(object)]
    pub struct ServerStatus {
        pub state: ConnectionState,
        pub steam_id: Option<BigInt>,
    }

    fn with_server<T>(f: impl FnOnce(&Dedicated) -> Result<T>) -> Result<T> {
        let server = SERVER.lock().unwrap();
        f(server
            .as_ref()
            .ok_or_else(|| Error::from_reason("Steam game server is not initialized"))?)
    }

    fn with_server_mut<T>(f: impl FnOnce(&mut Dedicated) -> Result<T>) -> Result<T> {
        let mut server = SERVER.lock().unwrap();
        f(server
            .as_mut()
            .ok_or_else(|| Error::from_reason("Steam game server is not initialized"))?)
    }

    fn bounded(name: &str, value: f64, min: u32, max: u32) -> Result<u32> {
        if value.fract() == 0.0 && (f64::from(min)..=f64::from(max)).contains(&value) {
            Ok(value as u32)
        } else {
            Err(Error::new(
                Status::InvalidArg,
                format!("{name} must be an integer from {min} to {max}"),
            ))
        }
    }

    #[napi(object)]
    pub struct ServerDiagnostics {
        pub app_id: u32,
        pub server_real_time: u32,
    }

    #[napi]
    pub fn diagnostics() -> Result<ServerDiagnostics> {
        with_server(|s| {
            if unsafe { steamworks::sys::SteamAPI_SteamGameServerUtils_v010() }.is_null() {
                return Err(Error::from_reason("Steam game server utils unavailable"));
            }
            let utils = s.sdk.utils();
            Ok(ServerDiagnostics {
                app_id: utils.app_id().0,
                server_real_time: utils.get_server_real_time(),
            })
        })
    }

    #[napi(object)]
    pub struct P2pState {
        pub connection_active: bool,
        pub connecting: bool,
        pub error: u32,
        pub using_relay: bool,
        pub bytes_queued_for_send: i32,
        pub packets_queued_for_send: i32,
        pub remote_ip: Option<String>,
        pub remote_port: Option<u32>,
    }

    #[napi(strict, js_name = "p2pSessionState")]
    pub fn p2p_session_state(id: BigInt) -> Result<Option<P2pState>> {
        let peer = session_peer(id)?;
        with_server(|s| {
            if unsafe { steamworks::sys::SteamAPI_SteamGameServerNetworking_v006() }.is_null() {
                return Err(Error::from_reason(
                    "Steam game server networking unavailable",
                ));
            }
            Ok(s.sdk
                .networking()
                .get_p2p_session_state(peer)
                .map(|state| P2pState {
                    connection_active: state.connection_active,
                    connecting: state.connecting,
                    error: u8::from(state.error).into(),
                    using_relay: state.using_relay,
                    bytes_queued_for_send: state.bytes_queued_for_send,
                    packets_queued_for_send: state.packets_queued_for_send,
                    remote_ip: state.remote_ip.map(|ip| ip.to_string()),
                    remote_port: state.remote_port.map(u32::from),
                }))
        })
    }

    #[napi(strict, js_name = "acceptP2pSession")]
    pub fn accept_p2p_session(id: BigInt) -> Result<bool> {
        let peer = session_peer(id)?;
        with_server(|s| {
            if unsafe { steamworks::sys::SteamAPI_SteamGameServerNetworking_v006() }.is_null() {
                return Err(Error::from_reason(
                    "Steam game server networking unavailable",
                ));
            }
            Ok(s.sdk.networking().accept_p2p_session(peer))
        })
    }

    #[napi(strict, js_name = "closeP2pSession")]
    pub fn close_p2p_session(id: BigInt) -> Result<bool> {
        let peer = session_peer(id)?;
        with_server(|s| {
            if unsafe { steamworks::sys::SteamAPI_SteamGameServerNetworking_v006() }.is_null() {
                return Err(Error::from_reason(
                    "Steam game server networking unavailable",
                ));
            }
            Ok(s.sdk.networking().close_p2p_session(peer))
        })
    }

    #[napi(object, object_from_js = false)]
    pub struct P2pSessionRequest {
        pub steam_id: BigInt,
    }

    #[napi(strict, js_name = "onP2pSessionRequest")]
    pub fn on_p2p_session_request(
        handler: Function<'_, P2pSessionRequest, ()>,
    ) -> Result<AuthListener> {
        let callback = handler.build_threadsafe_function().build()?;
        with_server_mut(|s| {
            s.p2p_listener.take();
            let id = LISTENER_ID.fetch_add(1, Ordering::Relaxed);
            let handle = s
                .sdk
                .register_callback(move |event: steamworks::P2PSessionRequest| {
                    callback.call(
                        P2pSessionRequest {
                            steam_id: event.remote.raw().into(),
                        },
                        ThreadsafeFunctionCallMode::NonBlocking,
                    );
                });
            s.p2p_listener = Some((id, handle));
            Ok(AuthListener { id, kind: 3 })
        })
    }

    fn messages_api() -> Result<*mut steamworks::sys::ISteamNetworkingMessages> {
        let api =
            unsafe { steamworks::sys::SteamAPI_SteamGameServerNetworkingMessages_SteamAPI_v002() };
        if api.is_null() {
            Err(Error::from_reason("Steam game server messages unavailable"))
        } else {
            Ok(api)
        }
    }

    fn message_identity(id: BigInt) -> Result<steamworks::sys::SteamNetworkingIdentity> {
        let peer = session_peer(id)?;
        let mut identity = unsafe { std::mem::zeroed() };
        unsafe {
            steamworks::sys::SteamAPI_SteamNetworkingIdentity_SetSteamID64(
                &mut identity,
                peer.raw(),
            )
        };
        Ok(identity)
    }

    #[napi(strict)]
    pub fn send_message_to_user(
        id: BigInt,
        data: Buffer,
        channel: f64,
        reliable: bool,
    ) -> Result<()> {
        let identity = message_identity(id)?;
        let channel = bounded("channel", channel, 0, i32::MAX as u32)?;
        if data.len() > 512 * 1024 {
            return Err(Error::new(Status::InvalidArg, "Message exceeds 512 KiB"));
        }
        with_server(|s| {
            messages_api()?;
            let flags = if reliable {
                steamworks::networking_types::SendFlags::RELIABLE
            } else {
                steamworks::networking_types::SendFlags::UNRELIABLE
            };
            s.sdk
                .networking_messages()
                .send_message_to_user(identity.into(), flags, &data, channel)
                .map_err(|e| Error::from_reason(format!("Failed to send Steam message: {e}")))
        })
    }

    #[napi(strict)]
    pub fn accept_message_session(id: BigInt) -> Result<bool> {
        let identity = message_identity(id)?;
        with_server(|_| {
            Ok(unsafe {
                steamworks::sys::SteamAPI_ISteamNetworkingMessages_AcceptSessionWithUser(
                    messages_api()?,
                    &identity,
                )
            })
        })
    }

    #[napi(strict)]
    pub fn close_message_session(id: BigInt) -> Result<bool> {
        let identity = message_identity(id)?;
        with_server(|_| {
            Ok(unsafe {
                steamworks::sys::SteamAPI_ISteamNetworkingMessages_CloseSessionWithUser(
                    messages_api()?,
                    &identity,
                )
            })
        })
    }

    #[napi(object, object_from_js = false)]
    pub struct MessageSessionRequest {
        pub steam_id: Option<BigInt>,
    }

    unsafe impl steamworks::Callback for MessageSessionRequest {
        const ID: i32 = steamworks::sys::SteamNetworkingMessagesSessionRequest_t_k_iCallback as i32;

        unsafe fn from_raw(raw: *mut std::ffi::c_void) -> Self {
            let event = raw.cast::<steamworks::sys::SteamNetworkingMessagesSessionRequest_t>();
            let mut identity = std::ptr::addr_of!((*event).m_identityRemote).read_unaligned();
            let id = steamworks::sys::SteamAPI_SteamNetworkingIdentity_GetSteamID64(&mut identity);
            Self {
                steam_id: (id != 0).then(|| id.into()),
            }
        }
    }

    #[napi(strict)]
    pub fn on_message_session_request(
        handler: Function<'_, MessageSessionRequest, ()>,
    ) -> Result<AuthListener> {
        let callback = handler.build_threadsafe_function().build()?;
        with_server_mut(|s| {
            s.message_listener.take();
            let id = LISTENER_ID.fetch_add(1, Ordering::Relaxed);
            let handle = s
                .sdk
                .register_callback(move |event: MessageSessionRequest| {
                    callback.call(event, ThreadsafeFunctionCallMode::NonBlocking);
                });
            s.message_listener = Some((id, handle));
            Ok(AuthListener { id, kind: 2 })
        })
    }

    struct ReceivedMessages(Vec<*mut steamworks::sys::SteamNetworkingMessage_t>);

    impl Drop for ReceivedMessages {
        fn drop(&mut self) {
            for message in &self.0 {
                if !message.is_null() {
                    unsafe { steamworks::sys::SteamAPI_SteamNetworkingMessage_t_Release(*message) }
                }
            }
        }
    }

    #[napi(object)]
    pub struct ServerMessage {
        pub data: Buffer,
        pub steam_id: Option<BigInt>,
    }

    fn collect_messages(messages: ReceivedMessages, count: i32) -> Result<Vec<ServerMessage>> {
        if count < 0 || count as usize > messages.0.len() {
            return Err(Error::from_reason(
                "Steam returned an invalid message count",
            ));
        }
        let mut out = Vec::with_capacity(count as usize);
        for message in messages.0.iter().take(count as usize) {
            if message.is_null() {
                return Err(Error::from_reason("Steam returned a null message"));
            }
            let raw = unsafe { &**message };
            if raw.m_cbSize < 0
                || raw.m_cbSize as usize > 512 * 1024
                || (raw.m_cbSize > 0 && raw.m_pData.is_null())
            {
                return Err(Error::from_reason(
                    "Steam returned an invalid message payload",
                ));
            }
            let data = if raw.m_cbSize == 0 {
                Vec::new()
            } else {
                unsafe {
                    std::slice::from_raw_parts(raw.m_pData.cast::<u8>(), raw.m_cbSize as usize)
                }
                .to_vec()
            };
            let steam_id =
                steamworks::networking_types::NetworkingIdentity::from(raw.m_identityPeer)
                    .steam_id()
                    .map(|id| id.raw().into());
            out.push(ServerMessage {
                data: data.into(),
                steam_id,
            });
        }
        Ok(out)
    }

    #[napi(strict)]
    pub fn receive_messages(channel: f64, max_messages: f64) -> Result<Vec<ServerMessage>> {
        let channel = bounded("channel", channel, 0, i32::MAX as u32)? as i32;
        let max = bounded("maxMessages", max_messages, 1, 32)? as usize;
        with_server(|_| {
            let mut messages = ReceivedMessages(vec![std::ptr::null_mut(); max]);
            let count = unsafe {
                steamworks::sys::SteamAPI_ISteamNetworkingMessages_ReceiveMessagesOnChannel(
                    messages_api()?,
                    channel,
                    messages.0.as_mut_ptr(),
                    max as i32,
                )
            };
            collect_messages(messages, count)
        })
    }

    #[napi(object)]
    pub struct HostedEvent {
        pub connection: BigInt,
        pub state: i32,
        pub steam_id: Option<BigInt>,
        pub end_reason: i32,
    }

    #[napi(object)]
    pub struct HostedEvents {
        pub events: Vec<HostedEvent>,
        pub dropped: u32,
    }

    struct RawHostedEvent {
        socket: u32,
        connection: u32,
        state: i32,
        steam_id: u64,
        end_reason: i32,
    }

    unsafe impl steamworks::Callback for RawHostedEvent {
        const ID: i32 =
            steamworks::sys::SteamNetConnectionStatusChangedCallback_t_k_iCallback as i32;

        unsafe fn from_raw(raw: *mut std::ffi::c_void) -> Self {
            let event = raw.cast::<steamworks::sys::SteamNetConnectionStatusChangedCallback_t>();
            let mut identity =
                std::ptr::addr_of!((*event).m_info.m_identityRemote).read_unaligned();
            Self {
                socket: std::ptr::addr_of!((*event).m_info.m_hListenSocket).read_unaligned(),
                connection: std::ptr::addr_of!((*event).m_hConn).read_unaligned(),
                state: std::ptr::addr_of!((*event).m_info.m_eState)
                    .cast::<i32>()
                    .read_unaligned(),
                steam_id: steamworks::sys::SteamAPI_SteamNetworkingIdentity_GetSteamID64(
                    &mut identity,
                ),
                end_reason: std::ptr::addr_of!((*event).m_info.m_eEndReason).read_unaligned(),
            }
        }
    }

    fn sockets_api() -> Result<*mut steamworks::sys::ISteamNetworkingSockets> {
        let api =
            unsafe { steamworks::sys::SteamAPI_SteamGameServerNetworkingSockets_SteamAPI_v012() };
        if api.is_null() {
            Err(Error::from_reason("Steam game server sockets unavailable"))
        } else {
            Ok(api)
        }
    }

    #[napi(strict)]
    pub fn open_hosted_listen_socket(virtual_port: f64) -> Result<()> {
        let port = bounded("virtualPort", virtual_port, 0, 65535)?;
        with_server_mut(|s| {
            if s.hosted.is_some() {
                return Err(Error::from_reason("Hosted listen socket is already open"));
            }
            let socket = unsafe {
                steamworks::sys::SteamAPI_ISteamNetworkingSockets_CreateHostedDedicatedServerListenSocket(
                    sockets_api()?, port as i32, 0, std::ptr::null(),
                )
            };
            if socket == 0 {
                return Err(Error::from_reason(
                    "Steam could not create the hosted listen socket",
                ));
            }
            let hosted = Arc::new(Mutex::new(Hosted {
                socket,
                connections: HashMap::new(),
                events: VecDeque::new(),
                dropped: 0,
            }));
            let target = Arc::clone(&hosted);
            let callback = s.sdk.register_callback(move |event: RawHostedEvent| {
                let mut hosted = target.lock().unwrap();
                if event.socket != hosted.socket || event.connection == 0 {
                    return;
                }
                let existing = hosted
                    .connections
                    .iter()
                    .find_map(|(id, raw)| (*raw == event.connection).then_some(*id));
                let id = match (existing, event.state) {
                    (Some(id), _) => id,
                    (None, 1) if hosted.connections.len() < 128 => {
                        let id = LISTENER_ID.fetch_add(1, Ordering::Relaxed);
                        hosted.connections.insert(id, event.connection);
                        id
                    }
                    (None, 1) => {
                        hosted.dropped = hosted.dropped.saturating_add(1);
                        if let Ok(sockets) = sockets_api() {
                            unsafe {
                                steamworks::sys::SteamAPI_ISteamNetworkingSockets_CloseConnection(
                                    sockets,
                                    event.connection,
                                    0,
                                    std::ptr::null(),
                                    false,
                                );
                            }
                        }
                        return;
                    }
                    _ => return,
                };
                if event.state == 4 || event.state == 5 {
                    hosted.connections.remove(&id);
                    if let Ok(sockets) = sockets_api() {
                        unsafe {
                            steamworks::sys::SteamAPI_ISteamNetworkingSockets_CloseConnection(
                                sockets,
                                event.connection,
                                0,
                                std::ptr::null(),
                                false,
                            );
                        }
                    }
                }
                if hosted.events.len() == 256 {
                    hosted.events.pop_front();
                    hosted.dropped = hosted.dropped.saturating_add(1);
                }
                hosted.events.push_back(HostedEvent {
                    connection: id.into(),
                    state: event.state,
                    steam_id: (event.steam_id != 0).then(|| event.steam_id.into()),
                    end_reason: event.end_reason,
                });
            });
            s.hosted = Some(hosted);
            s.hosted_callback = Some(callback);
            Ok(())
        })
    }

    #[napi]
    pub fn close_hosted_listen_socket() -> Result<bool> {
        with_server_mut(|s| Ok(s.close_hosted()))
    }

    #[napi(strict)]
    pub fn take_hosted_events(max_events: f64) -> Result<HostedEvents> {
        let max = bounded("maxEvents", max_events, 1, 128)? as usize;
        with_server(|s| {
            let mut hosted = s
                .hosted
                .as_ref()
                .ok_or_else(|| Error::from_reason("Hosted listen socket is not open"))?
                .lock()
                .unwrap();
            let count = max.min(hosted.events.len());
            let events = hosted.events.drain(..count).collect();
            let dropped = std::mem::take(&mut hosted.dropped);
            Ok(HostedEvents { events, dropped })
        })
    }

    fn hosted_connection(s: &Dedicated, connection: BigInt) -> Result<u32> {
        let (_, id, exact) = connection.get_u64();
        if !exact || id == 0 {
            return Err(Error::new(
                Status::InvalidArg,
                "Invalid hosted connection token",
            ));
        }
        let hosted = s
            .hosted
            .as_ref()
            .ok_or_else(|| Error::from_reason("Hosted listen socket is not open"))?
            .lock()
            .unwrap();
        hosted
            .connections
            .get(&id)
            .copied()
            .ok_or_else(|| Error::from_reason("Unknown hosted connection"))
    }

    #[napi(strict)]
    pub fn accept_hosted_connection(connection: BigInt) -> Result<()> {
        with_server(|s| {
            let raw = hosted_connection(s, connection)?;
            let result = unsafe {
                steamworks::sys::SteamAPI_ISteamNetworkingSockets_AcceptConnection(
                    sockets_api()?,
                    raw,
                )
            };
            if result == steamworks::sys::EResult::k_EResultOK {
                Ok(())
            } else {
                Err(Error::from_reason(format!(
                    "AcceptConnection returned {result:?}"
                )))
            }
        })
    }

    #[napi(strict)]
    pub fn send_hosted_message(connection: BigInt, data: Buffer, reliable: bool) -> Result<BigInt> {
        if data.len() > 512 * 1024 {
            return Err(Error::new(Status::InvalidArg, "Message exceeds 512 KiB"));
        }
        with_server(|s| {
            let raw = hosted_connection(s, connection)?;
            let flags = if reliable {
                steamworks::sys::k_nSteamNetworkingSend_Reliable
            } else {
                steamworks::sys::k_nSteamNetworkingSend_Unreliable
            };
            let mut number = 0i64;
            let result = unsafe {
                steamworks::sys::SteamAPI_ISteamNetworkingSockets_SendMessageToConnection(
                    sockets_api()?,
                    raw,
                    data.as_ptr().cast(),
                    data.len() as u32,
                    flags,
                    &mut number,
                )
            };
            if result != steamworks::sys::EResult::k_EResultOK {
                return Err(Error::from_reason(format!(
                    "SendMessageToConnection returned {result:?}"
                )));
            }
            if number < 0 {
                return Err(Error::from_reason(
                    "Steam returned an invalid message number",
                ));
            }
            Ok((number as u64).into())
        })
    }

    #[napi(strict)]
    pub fn receive_hosted_messages(
        connection: BigInt,
        max_messages: f64,
    ) -> Result<Vec<ServerMessage>> {
        let max = bounded("maxMessages", max_messages, 1, 32)? as usize;
        with_server(|s| {
            let raw = hosted_connection(s, connection)?;
            let mut messages = ReceivedMessages(vec![std::ptr::null_mut(); max]);
            let count = unsafe {
                steamworks::sys::SteamAPI_ISteamNetworkingSockets_ReceiveMessagesOnConnection(
                    sockets_api()?,
                    raw,
                    messages.0.as_mut_ptr(),
                    max as i32,
                )
            };
            collect_messages(messages, count)
        })
    }

    #[napi(strict)]
    pub fn close_hosted_connection(connection: BigInt) -> Result<bool> {
        with_server(|s| {
            let raw = hosted_connection(s, connection)?;
            let success = unsafe {
                steamworks::sys::SteamAPI_ISteamNetworkingSockets_CloseConnection(
                    sockets_api()?,
                    raw,
                    0,
                    std::ptr::null(),
                    false,
                )
            };
            if success {
                let mut hosted = s.hosted.as_ref().unwrap().lock().unwrap();
                hosted.connections.retain(|_, value| *value != raw);
            }
            Ok(success)
        })
    }

    fn text(name: &str, value: &str, max: usize) -> Result<()> {
        if value.trim().is_empty() || value.len() > max || value.contains('\0') {
            Err(Error::new(
                Status::InvalidArg,
                format!("{name} must be nonblank, NUL-free and at most {max} UTF-8 bytes"),
            ))
        } else {
            Ok(())
        }
    }

    fn number(name: &str, value: f64, max: u16) -> Result<u16> {
        if value.fract() != 0.0 || !(1.0..=f64::from(max)).contains(&value) {
            return Err(Error::new(
                Status::InvalidArg,
                format!("{name} must be an integer from 1 to {max}"),
            ));
        }
        Ok(value as u16)
    }

    impl ServerListing {
        fn validate(&self) -> Result<(u16, Option<u16>)> {
            text("serverName", &self.server_name, 63)?;
            text("mapName", &self.map_name, 31)?;
            let max = number("maxPlayers", self.max_players, 255)?;
            let bots = self
                .bot_players
                .map(|value| {
                    if value.fract() != 0.0 || !(0.0..=f64::from(max)).contains(&value) {
                        return Err(Error::new(
                            Status::InvalidArg,
                            "botPlayers must be an integer from 0 to maxPlayers",
                        ));
                    }
                    Ok(value as u16)
                })
                .transpose()?;
            Ok((max, bots))
        }

        fn apply(&self, server: &Server, max_players: u16, bots: Option<u16>) {
            server.set_server_name(&self.server_name);
            server.set_map_name(&self.map_name);
            server.set_max_players(i32::from(max_players));
            if let Some(bots) = bots {
                server.set_bot_player_count(i32::from(bots));
            }
            if let Some(protected) = self.password_protected {
                server.set_password_protected(protected);
            }
        }
    }

    #[napi(strict)]
    pub fn init(options: ServerOptions) -> Result<()> {
        let ip: Ipv4Addr = options
            .bind_ip
            .parse()
            .map_err(|_| Error::new(Status::InvalidArg, "bindIp must be a dotted IPv4 address"))?;
        let game_port = number("gamePort", options.game_port, u16::MAX)?;
        let query_port = number("queryPort", options.query_port, u16::MAX)?;
        let shared_socket = query_port == steamworks::QUERY_PORT_SHARED;
        if !shared_socket && game_port == query_port {
            return Err(Error::new(
                Status::InvalidArg,
                "gamePort and queryPort must differ",
            ));
        }
        text("version", &options.version, 63)?;
        text("product", &options.product, 63)?;
        text("description", &options.description, 63)?;
        let (max_players, bots) = options.listing.validate()?;
        let mode = match options.mode {
            Mode::NoAuthentication => ServerMode::NoAuthentication,
            Mode::Authentication => ServerMode::Authentication,
            Mode::AuthenticationAndSecure => ServerMode::AuthenticationAndSecure,
        };
        crate::process_mode::initialize(crate::process_mode::Mode::Server, || {
            let mut slot = SERVER.lock().unwrap();
            if slot.is_some() {
                return Err(Error::from_reason(
                    "Steam game server is already initialized",
                ));
            }
            let (sdk, _) = Server::init(ip, game_port, query_port, mode, &options.version)
                .map_err(|e| Error::from_reason(format!("Steam game server init failed: {e:?}")))?;
            sdk.set_dedicated_server(true);
            sdk.set_product(&options.product);
            sdk.set_game_description(&options.description);
            options.listing.apply(&sdk, max_players, bots);
            sdk.set_advertise_server_active(false);
            let phase = Arc::new(Mutex::new(ConnectionState::Ready));
            let connected = phase.clone();
            let failed = phase.clone();
            let disconnected = phase.clone();
            let callbacks = [
                sdk.register_callback(move |_: SteamServersConnected| {
                    *connected.lock().unwrap() = ConnectionState::Connected
                }),
                sdk.register_callback(move |event: SteamServerConnectFailure| {
                    *failed.lock().unwrap() = if event.still_retrying {
                        ConnectionState::Connecting
                    } else {
                        ConnectionState::Failed
                    }
                }),
                sdk.register_callback(move |_: SteamServersDisconnected| {
                    *disconnected.lock().unwrap() = ConnectionState::Disconnected
                }),
            ];
            *slot = Some(Dedicated {
                sdk,
                phase,
                callbacks: Some(callbacks),
                auth_listener: None,
                server_listener: None,
                message_listener: None,
                p2p_listener: None,
                shared_socket,
                hosted: None,
                hosted_callback: None,
            });
            Ok(())
        })
    }

    #[napi(strict)]
    pub fn update_listing(listing: ServerListing) -> Result<()> {
        let (max_players, bots) = listing.validate()?;
        with_server(|s| {
            listing.apply(&s.sdk, max_players, bots);
            Ok(())
        })
    }

    #[napi(strict)]
    pub fn set_advertising(active: bool) -> Result<()> {
        with_server(|s| {
            s.sdk.set_advertise_server_active(active);
            Ok(())
        })
    }

    #[napi(strict)]
    pub fn set_game_data(data: String) -> Result<()> {
        if data.len() > 2047 || data.contains('\0') {
            return Err(Error::new(
                Status::InvalidArg,
                "game data must be NUL-free and at most 2047 UTF-8 bytes",
            ));
        }
        with_server(|s| {
            s.sdk.set_game_data(&data);
            Ok(())
        })
    }

    #[napi(strict)]
    pub fn set_mod_dir(directory: String) -> Result<()> {
        if directory.len() > 31 || directory.contains('\0') {
            return Err(Error::new(
                Status::InvalidArg,
                "mod directory must be NUL-free and at most 31 UTF-8 bytes",
            ));
        }
        with_server(|s| {
            s.sdk.set_mod_dir(&directory);
            Ok(())
        })
    }

    #[napi(strict)]
    pub fn set_game_tags(tags: String) -> Result<()> {
        text("tags", &tags, 127)?;
        with_server(|s| {
            s.sdk.set_game_tags(&tags);
            Ok(())
        })
    }

    #[napi(strict)]
    pub fn set_key_value(key: String, value: String) -> Result<()> {
        text("key", &key, 255)?;
        if value.len() > 255 || value.contains('\0') {
            return Err(Error::new(
                Status::InvalidArg,
                "value must be NUL-free and at most 255 UTF-8 bytes",
            ));
        }
        with_server(|s| {
            s.sdk.set_key_value(&key, &value);
            Ok(())
        })
    }

    #[napi]
    pub fn clear_all_key_values() -> Result<()> {
        with_server(|s| {
            s.sdk.clear_all_key_values();
            Ok(())
        })
    }

    #[napi(object)]
    pub struct OutgoingPacket {
        pub address: String,
        pub port: u32,
        pub data: Buffer,
    }

    #[napi(strict)]
    pub fn handle_incoming_packet(data: Buffer, address: String, port: f64) -> Result<bool> {
        if data.is_empty() || data.len() > 16384 {
            return Err(Error::new(
                Status::InvalidArg,
                "packet must contain 1 to 16384 bytes",
            ));
        }
        let ip: Ipv4Addr = address
            .parse()
            .map_err(|_| Error::new(Status::InvalidArg, "address must be a dotted IPv4 address"))?;
        let port = number("port", port, u16::MAX)?;
        with_server(|s| {
            if !s.shared_socket {
                return Err(Error::from_reason("Shared socket mode is not enabled"));
            }
            Ok(s.sdk
                .handle_incoming_packet(&data, SocketAddrV4::new(ip, port)))
        })
    }

    #[napi]
    pub fn get_next_outgoing_packet() -> Result<Option<OutgoingPacket>> {
        with_server(|s| {
            if !s.shared_socket {
                return Err(Error::from_reason("Shared socket mode is not enabled"));
            }
            let mut data = vec![0u8; 16384];
            let mut ip = 0u32;
            let mut port = 0u16;
            let len = unsafe {
                steamworks::sys::SteamAPI_ISteamGameServer_GetNextOutgoingPacket(
                    steamworks::sys::SteamAPI_SteamGameServer_v015(),
                    data.as_mut_ptr().cast(),
                    data.len() as i32,
                    &mut ip,
                    &mut port,
                )
            };
            if len == 0 {
                return Ok(None);
            }
            if len < 0 || len as usize > data.len() {
                return Err(Error::from_reason(
                    "Steam returned an invalid packet length",
                ));
            }
            data.truncate(len as usize);
            Ok(Some(OutgoingPacket {
                address: Ipv4Addr::from(ip).to_string(),
                port: u32::from(port),
                data: data.into(),
            }))
        })
    }

    #[napi]
    pub struct AuthListener {
        id: u64,
        kind: u8,
    }

    #[napi]
    impl AuthListener {
        #[napi]
        pub fn disconnect(&mut self) {
            let Ok(mut slot) = SERVER.lock() else { return };
            if let Some(server) = slot.as_mut() {
                if self.kind == 1 {
                    if server
                        .server_listener
                        .as_ref()
                        .is_some_and(|(id, _)| *id == self.id)
                    {
                        server.server_listener.take();
                    }
                } else if self.kind == 3 {
                    if server
                        .p2p_listener
                        .as_ref()
                        .is_some_and(|(id, _)| *id == self.id)
                    {
                        server.p2p_listener.take();
                    }
                } else if self.kind == 2 {
                    if server
                        .message_listener
                        .as_ref()
                        .is_some_and(|(id, _)| *id == self.id)
                    {
                        server.message_listener.take();
                    }
                } else if server
                    .auth_listener
                    .as_ref()
                    .is_some_and(|(id, _)| *id == self.id)
                {
                    server.auth_listener.take();
                }
            }
        }
    }

    fn session_peer(id: BigInt) -> Result<SteamId> {
        let (_, raw, lossless) = id.get_u64();
        if !lossless || raw == 0 {
            return Err(Error::new(
                Status::InvalidArg,
                "Expected a nonzero unsigned 64-bit Steam ID",
            ));
        }
        Ok(SteamId::from_raw(raw))
    }

    #[napi(strict)]
    pub fn begin_auth_session(steam_id64: BigInt, ticket: Buffer) -> Result<()> {
        let peer = session_peer(steam_id64)?;
        if ticket.is_empty() || ticket.len() > i32::MAX as usize {
            return Err(Error::new(
                Status::InvalidArg,
                "Expected a nonempty ticket fitting a signed 32-bit length",
            ));
        }
        with_server(|s| {
            s.sdk
                .begin_authentication_session(peer, &ticket)
                .map_err(|error| Error::from_reason(error.to_string()))
        })
    }

    #[napi(strict)]
    pub fn end_auth_session(steam_id64: BigInt) -> Result<()> {
        let peer = session_peer(steam_id64)?;
        with_server(|s| {
            s.sdk.end_authentication_session(peer);
            Ok(())
        })
    }

    #[napi(strict)]
    pub fn request_user_group_status(user: BigInt, group: BigInt) -> Result<bool> {
        let user = session_peer(user)?.raw();
        let group = session_peer(group)?.raw();
        with_server(|_| {
            Ok(unsafe {
                steamworks::sys::SteamAPI_ISteamGameServer_RequestUserGroupStatus(
                    steamworks::sys::SteamAPI_SteamGameServer_v015(),
                    user,
                    group,
                )
            })
        })
    }

    #[napi(strict)]
    pub fn on_validate_auth_ticket_response(
        #[napi(
            ts_arg_type = "(event: { steam_id: bigint; owner_steam_id: bigint; response: number }) => void"
        )]
        handler: Function<'_, TicketValidation, ()>,
    ) -> Result<AuthListener> {
        let listener = handler.build_threadsafe_function().build()?;
        let mut slot = SERVER.lock().unwrap();
        let s = slot
            .as_mut()
            .ok_or_else(|| Error::from_reason("Steam game server is not initialized"))?;
        s.auth_listener.take();
        let id = LISTENER_ID.fetch_add(1, Ordering::Relaxed);
        let handle = s.sdk.register_callback(move |event: TicketValidation| {
            listener.call(event, ThreadsafeFunctionCallMode::NonBlocking);
        });
        s.auth_listener = Some((id, handle));
        Ok(AuthListener { id, kind: 0 })
    }

    #[napi(object, object_from_js = false)]
    pub struct ServerEvent {
        pub kind: String,
        pub user: BigInt,
        pub owner: Option<BigInt>,
        pub group: Option<BigInt>,
        pub reason: Option<i32>,
        pub text: Option<String>,
        pub member: Option<bool>,
        pub officer: Option<bool>,
    }

    #[napi(strict)]
    pub fn on_server_event(
        #[napi(
            ts_arg_type = "(event: { kind: 'approve'; user: bigint; owner: bigint } | { kind: 'deny'; user: bigint; reason: number; text: string } | { kind: 'kick'; user: bigint; reason: number } | { kind: 'group'; user: bigint; group: bigint; member: boolean; officer: boolean }) => void"
        )]
        handler: Function<'_, ServerEvent, ()>,
    ) -> Result<AuthListener> {
        let listener = Arc::new(handler.build_threadsafe_function().build()?);
        let mut slot = SERVER.lock().unwrap();
        let s = slot
            .as_mut()
            .ok_or_else(|| Error::from_reason("Steam game server is not initialized"))?;
        s.server_listener.take();
        let id = LISTENER_ID.fetch_add(1, Ordering::Relaxed);
        let approve = listener.clone();
        let deny = listener.clone();
        let kick = listener.clone();
        let handles = vec![
            s.sdk.register_callback(move |e: GSClientApprove| {
                approve.call(
                    ServerEvent {
                        kind: "approve".into(),
                        user: e.user.raw().into(),
                        owner: Some(e.owner.raw().into()),
                        group: None,
                        reason: None,
                        text: None,
                        member: None,
                        officer: None,
                    },
                    ThreadsafeFunctionCallMode::NonBlocking,
                );
            }),
            s.sdk.register_callback(move |e: GSClientDeny| {
                deny.call(
                    ServerEvent {
                        kind: "deny".into(),
                        user: e.user.raw().into(),
                        owner: None,
                        group: None,
                        reason: Some(e.deny_reason as i32),
                        text: Some(e.optional_text),
                        member: None,
                        officer: None,
                    },
                    ThreadsafeFunctionCallMode::NonBlocking,
                );
            }),
            s.sdk.register_callback(move |e: GSClientKick| {
                kick.call(
                    ServerEvent {
                        kind: "kick".into(),
                        user: e.user.raw().into(),
                        owner: None,
                        group: None,
                        reason: Some(e.deny_reason as i32),
                        text: None,
                        member: None,
                        officer: None,
                    },
                    ThreadsafeFunctionCallMode::NonBlocking,
                );
            }),
            s.sdk.register_callback(move |e: GSClientGroupStatus| {
                listener.call(
                    ServerEvent {
                        kind: "group".into(),
                        user: e.user.raw().into(),
                        owner: None,
                        group: Some(e.group.raw().into()),
                        reason: None,
                        text: None,
                        member: Some(e.member),
                        officer: Some(e.officer),
                    },
                    ThreadsafeFunctionCallMode::NonBlocking,
                );
            }),
        ];
        s.server_listener = Some((id, handles));
        Ok(AuthListener { id, kind: 1 })
    }

    fn start_logon(submit: impl FnOnce(&Server)) -> Result<()> {
        with_server(|s| {
            let mut phase = s.phase.lock().unwrap();
            if !matches!(*phase, ConnectionState::Ready) {
                return Err(Error::from_reason(
                    "Logon already requested for this game server",
                ));
            }
            submit(&s.sdk);
            *phase = ConnectionState::Connecting;
            Ok(())
        })
    }

    #[napi(strict)]
    pub fn log_on(token: String) -> Result<()> {
        text("token", &token, 1024)?;
        start_logon(|sdk| sdk.log_on(&token))
    }

    #[napi]
    pub fn log_on_anonymous() -> Result<()> {
        start_logon(Server::log_on_anonymous)
    }

    #[napi]
    pub fn status() -> Result<ServerStatus> {
        with_server(|s| {
            let state = *s.phase.lock().unwrap();
            let steam_id = matches!(state, ConnectionState::Connected)
                .then(|| BigInt::from(s.sdk.steam_id().raw()));
            Ok(ServerStatus { state, steam_id })
        })
    }

    #[napi]
    pub fn run_callbacks() -> Result<()> {
        with_server(|s| {
            s.sdk.run_callbacks();
            Ok(())
        })
    }

    #[napi]
    pub fn shutdown() -> bool {
        let mut slot = SERVER.lock().unwrap();
        slot.take().is_some()
    }
}
