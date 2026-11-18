use napi::{Error, Result};
use napi_derive::napi;
use std::sync::{Arc, Mutex};
use steamworks::{CallbackHandle, Client, Server, ServerManager};

struct Dedicated {
    sdk: Server,
    pump: Client<ServerManager>,
    phase: Arc<Mutex<game_server::ConnectionState>>,
    _callbacks: [CallbackHandle<ServerManager>; 3],
}

static SERVER: Mutex<Option<Dedicated>> = Mutex::new(None);

#[napi]
pub mod game_server {
    use super::*;
    use crate::api::steam_events::TicketValidation;
    use napi::{
        bindgen_prelude::{BigInt, Buffer, Function},
        threadsafe_function::ThreadsafeFunctionCallMode,
        Status,
    };
    use std::net::Ipv4Addr;
    use steamworks::{
        ServerMode, SteamId, SteamServerConnectFailure, SteamServersConnected,
        SteamServersDisconnected,
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
        fn validate(&self) -> Result<u16> {
            text("serverName", &self.server_name, 63)?;
            text("mapName", &self.map_name, 31)?;
            number("maxPlayers", self.max_players, 255)
        }

        fn apply(&self, server: &Server, max_players: u16) {
            server.set_server_name(&self.server_name);
            server.set_map_name(&self.map_name);
            server.set_max_players(i32::from(max_players));
        }
    }

    #[napi(strict)]
    pub fn init(options: ServerOptions) -> Result<()> {
        let ip: Ipv4Addr = options
            .bind_ip
            .parse()
            .map_err(|_| Error::new(Status::InvalidArg, "bindIp must be a dotted IPv4 address"))?;
        let game_port = number("gamePort", options.game_port, u16::MAX)?;
        // 65535 enables shared-socket mode, which requires a packet forwarding API.
        let query_port = number("queryPort", options.query_port, u16::MAX - 1)?;
        if game_port == query_port {
            return Err(Error::new(
                Status::InvalidArg,
                "gamePort and queryPort must differ",
            ));
        }
        text("version", &options.version, 63)?;
        text("product", &options.product, 63)?;
        text("description", &options.description, 63)?;
        let max_players = options.listing.validate()?;
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
            let (sdk, pump) = Server::init(ip, game_port, query_port, mode, &options.version)
                .map_err(|e| Error::from_reason(format!("Steam game server init failed: {e:?}")))?;
            sdk.set_dedicated_server(true);
            sdk.set_product(&options.product);
            sdk.set_game_description(&options.description);
            options.listing.apply(&sdk, max_players);
            sdk.enable_heartbeats(false);
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
                pump,
                phase,
                _callbacks: callbacks,
            });
            Ok(())
        })
    }

    #[napi(strict)]
    pub fn update_listing(listing: ServerListing) -> Result<()> {
        let max_players = listing.validate()?;
        with_server(|s| {
            listing.apply(&s.sdk, max_players);
            Ok(())
        })
    }

    #[napi(strict)]
    pub fn set_advertising(active: bool) -> Result<()> {
        with_server(|s| {
            s.sdk.enable_heartbeats(active);
            Ok(())
        })
    }

    #[napi]
    pub struct AuthListener {
        handle: Option<CallbackHandle<ServerManager>>,
    }

    #[napi]
    impl AuthListener {
        #[napi]
        pub fn disconnect(&mut self) {
            if let Some(handle) = self.handle.take() {
                handle.disconnect();
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

    /// Submit a remote peer's ticket. Wait for the validation event before granting access.
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

    /// End a session when its peer disconnects, validation fails, or you abandon it.
    #[napi(strict)]
    pub fn end_auth_session(steam_id64: BigInt) -> Result<()> {
        let peer = session_peer(steam_id64)?;
        with_server(|s| {
            s.sdk.end_authentication_session(peer);
            Ok(())
        })
    }

    /// The game server's ticket validation event. One listener; a later call replaces it.
    #[napi(strict)]
    pub fn on_validate_auth_ticket_response(
        #[napi(
            ts_arg_type = "(event: { steam_id: bigint; owner_steam_id: bigint; response: number }) => void"
        )]
        handler: Function<'_, TicketValidation, ()>,
    ) -> Result<AuthListener> {
        let listener = handler.build_threadsafe_function().build()?;
        with_server(|s| {
            let handle = s.sdk.register_callback(move |event: TicketValidation| {
                listener.call(event, ThreadsafeFunctionCallMode::Blocking);
            });
            Ok(AuthListener {
                handle: Some(handle),
            })
        })
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
            s.pump.run_callbacks();
            Ok(())
        })
    }

    #[napi]
    pub fn shutdown() -> bool {
        let mut slot = SERVER.lock().unwrap();
        slot.take().is_some()
    }
}
