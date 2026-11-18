use napi_derive::napi;

#[napi]
pub mod networking_sockets {
    use napi::{
        bindgen_prelude::{BigInt, Buffer},
        threadsafe_function::{ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode},
        Env, Error, JsFunction, Result,
    };
    use std::{mem::MaybeUninit, net::IpAddr, ptr, slice};
    use steamworks::{networking_types::NetworkingIdentity, sys, Callback};

    fn sockets() -> Result<*mut sys::ISteamNetworkingSockets> {
        let api = unsafe { sys::SteamAPI_SteamNetworkingSockets_SteamAPI_v012() };
        if api.is_null() {
            Err(Error::from_reason("Steam Networking Sockets unavailable"))
        } else {
            Ok(api)
        }
    }

    fn integer(value: f64, min: u32, max: u32, name: &str) -> Result<u32> {
        if value.fract() == 0.0 && (min as f64..=max as f64).contains(&value) {
            Ok(value as u32)
        } else {
            Err(Error::from_reason(format!(
                "{name} must be an integer from {min} to {max}"
            )))
        }
    }

    fn handle(value: f64) -> Result<u32> {
        integer(value, 1, u32::MAX, "handle")
    }

    fn created(handle: u32, name: &str) -> Result<u32> {
        if handle == 0 {
            Err(Error::from_reason(format!(
                "{name} returned an invalid handle"
            )))
        } else {
            Ok(handle)
        }
    }

    fn parse_ip(address: &str) -> Result<IpAddr> {
        address
            .parse()
            .map_err(|_| Error::from_reason("Expected a literal IPv4 or IPv6 address"))
    }

    fn socket_address(ip: IpAddr, port: f64) -> Result<sys::SteamNetworkingIPAddr> {
        let port = integer(port, 1, 65535, "port")? as u16;
        unsafe {
            let mut addr = std::mem::zeroed();
            match ip {
                IpAddr::V4(v4) => {
                    sys::SteamAPI_SteamNetworkingIPAddr_SetIPv4(&mut addr, v4.into(), port)
                }
                IpAddr::V6(v6) => sys::SteamAPI_SteamNetworkingIPAddr_SetIPv6(
                    &mut addr,
                    v6.octets().as_ptr(),
                    port,
                ),
            }
            Ok(addr)
        }
    }

    #[cfg(test)]
    #[test]
    fn socket_address_preserves_network_bytes_and_host_port() {
        for ip in ["192.0.2.123", "2001:db8:abcd::1234"] {
            let ip = parse_ip(ip).unwrap();
            let addr = socket_address(ip, 31457.0).unwrap();
            let expected = match ip {
                IpAddr::V4(v4) => v4.to_ipv6_mapped(),
                IpAddr::V6(v6) => v6,
            };
            let port = addr.m_port;
            assert_eq!(port, 31457);
            assert_eq!(unsafe { addr.__bindgen_anon_1.m_ipv6 }, expected.octets());
        }
    }

    #[napi(strict)]
    pub fn create_listen_socket_ip(port: f64, address: Option<String>) -> Result<u32> {
        let addr = socket_address(parse_ip(address.as_deref().unwrap_or("127.0.0.1"))?, port)?;
        created(
            unsafe {
                sys::SteamAPI_ISteamNetworkingSockets_CreateListenSocketIP(
                    sockets()?,
                    &addr,
                    0,
                    ptr::null(),
                )
            },
            "CreateListenSocketIP",
        )
    }

    #[napi(strict)]
    pub fn connect_by_ip_address(address: String, port: f64) -> Result<u32> {
        let ip = parse_ip(&address)?;
        if ip.to_canonical().is_unspecified() {
            return Err(Error::from_reason(
                "Expected a non-wildcard destination address",
            ));
        }
        let addr = socket_address(ip, port)?;
        created(
            unsafe {
                sys::SteamAPI_ISteamNetworkingSockets_ConnectByIPAddress(
                    sockets()?,
                    &addr,
                    0,
                    ptr::null(),
                )
            },
            "ConnectByIPAddress",
        )
    }

    #[napi(strict)]
    pub fn create_listen_socket_p2p(virtual_port: f64) -> Result<u32> {
        let port = integer(virtual_port, 0, 65535, "virtualPort")?;
        created(
            unsafe {
                sys::SteamAPI_ISteamNetworkingSockets_CreateListenSocketP2P(
                    sockets()?,
                    port as i32,
                    0,
                    ptr::null(),
                )
            },
            "CreateListenSocketP2P",
        )
    }

    #[napi(strict)]
    pub fn connect_p2p(steam_id64: BigInt, virtual_port: f64) -> Result<u32> {
        let port = integer(virtual_port, 0, 65535, "virtualPort")?;
        let (negative, id, exact) = steam_id64.get_u64();
        if negative
            || !exact
            || !(1..=4).contains(&(id >> 56))
            || (id >> 52) & 15 != 1
            || (id >> 32) & 0xfffff != 1
            || id as u32 == 0
        {
            return Err(Error::from_reason("Expected a user SteamID64"));
        }
        let api = sockets()?;
        unsafe {
            let mut identity = std::mem::zeroed();
            sys::SteamAPI_SteamNetworkingIdentity_SetSteamID64(&mut identity, id);
            created(
                sys::SteamAPI_ISteamNetworkingSockets_ConnectP2P(
                    api,
                    &identity,
                    port as i32,
                    0,
                    ptr::null(),
                ),
                "ConnectP2P",
            )
        }
    }

    #[napi(object)]
    pub struct ConnectionInfo {
        pub connection: u32,
        pub listen_socket: u32,
        pub steam_id64: Option<BigInt>,
        pub state: i32,
        pub end_reason: i32,
    }

    fn info(connection: u32, raw: sys::SteamNetConnectionInfo_t) -> ConnectionInfo {
        ConnectionInfo {
            connection,
            listen_socket: raw.m_hListenSocket,
            state: raw.m_eState as i32,
            steam_id64: NetworkingIdentity::from(raw.m_identityRemote)
                .steam_id()
                .map(|id| id.raw().into()),
            end_reason: raw.m_eEndReason,
        }
    }

    unsafe impl Callback for ConnectionInfo {
        const ID: i32 = sys::SteamNetConnectionStatusChangedCallback_t_k_iCallback as i32;
        const SIZE: i32 =
            std::mem::size_of::<sys::SteamNetConnectionStatusChangedCallback_t>() as i32;
        unsafe fn from_raw(raw: *mut std::ffi::c_void) -> Self {
            let event = raw
                .cast::<sys::SteamNetConnectionStatusChangedCallback_t>()
                .read_unaligned();
            info(event.m_hConn, event.m_info)
        }
    }

    #[napi(strict)]
    pub fn set_connection_status_handler(
        env: Env,
        #[napi(ts_arg_type = "(info: ConnectionInfo) => void")] handler: Option<JsFunction>,
    ) -> Result<()> {
        let client = crate::client::get_client();
        if let Some(handler) = handler {
            let mut callback: ThreadsafeFunction<ConnectionInfo, ErrorStrategy::Fatal> =
                handler.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;
            callback.unref(&env)?;
            client.register_callback(move |event: ConnectionInfo| {
                callback.call(event, ThreadsafeFunctionCallMode::NonBlocking);
            });
        } else {
            client.register_callback(|_: ConnectionInfo| {});
        }
        Ok(())
    }

    #[napi(strict)]
    pub fn connection_info(connection: f64) -> Result<Option<ConnectionInfo>> {
        let conn = handle(connection)?;
        let mut out = MaybeUninit::zeroed();
        let ok = unsafe {
            sys::SteamAPI_ISteamNetworkingSockets_GetConnectionInfo(
                sockets()?,
                conn,
                out.as_mut_ptr(),
            )
        };
        Ok(ok.then(|| info(conn, unsafe { out.assume_init() })))
    }

    fn check(result: sys::EResult, operation: &str) -> Result<()> {
        if result == sys::EResult::k_EResultOK {
            Ok(())
        } else {
            Err(Error::from_reason(format!(
                "{operation} returned {result:?}"
            )))
        }
    }

    #[napi(strict)]
    pub fn accept_connection(connection: f64) -> Result<()> {
        let conn = handle(connection)?;
        check(
            unsafe { sys::SteamAPI_ISteamNetworkingSockets_AcceptConnection(sockets()?, conn) },
            "AcceptConnection",
        )
    }

    #[napi(strict)]
    pub fn send_message(connection: f64, data: Buffer, reliable: Option<bool>) -> Result<BigInt> {
        let conn = handle(connection)?;
        if data.len() > 512 * 1024 {
            return Err(Error::from_reason("Message exceeds 512 KiB"));
        }
        let flags = if reliable.unwrap_or(true) {
            sys::k_nSteamNetworkingSend_Reliable
        } else {
            sys::k_nSteamNetworkingSend_Unreliable
        };
        let mut number = 0;
        check(
            unsafe {
                sys::SteamAPI_ISteamNetworkingSockets_SendMessageToConnection(
                    sockets()?,
                    conn,
                    data.as_ptr().cast(),
                    data.len() as u32,
                    flags,
                    &mut number,
                )
            },
            "SendMessageToConnection",
        )?;
        Ok(number.into())
    }

    const MAX_BATCH: usize = 32;
    struct Batch([*mut sys::SteamNetworkingMessage_t; MAX_BATCH]);

    impl Drop for Batch {
        fn drop(&mut self) {
            for &message in self.0.iter().filter(|p| !p.is_null()) {
                unsafe { sys::SteamAPI_SteamNetworkingMessage_t_Release(message) };
            }
        }
    }

    fn receive<T>(
        target: u32,
        max_messages: Option<f64>,
        read: unsafe extern "C" fn(
            *mut sys::ISteamNetworkingSockets,
            u32,
            *mut *mut sys::SteamNetworkingMessage_t,
            i32,
        ) -> i32,
        wrap: impl Fn(u32, Buffer) -> T,
    ) -> Result<Vec<T>> {
        let limit = integer(
            max_messages.unwrap_or(10.0),
            1,
            MAX_BATCH as u32,
            "maxMessages",
        )?;
        let mut batch = Batch([ptr::null_mut(); MAX_BATCH]);
        let n = unsafe { read(sockets()?, target, batch.0.as_mut_ptr(), limit as i32) };
        if !(0..=limit as i32).contains(&n) {
            return Err(Error::from_reason("Steam socket receive failed"));
        }
        batch.0[..n as usize]
            .iter()
            .map(|&p| unsafe {
                let message = p
                    .as_ref()
                    .ok_or_else(|| Error::from_reason("Missing Steam message"))?;
                let len = message.m_cbSize;
                if len == 0 {
                    return Ok(wrap(message.m_conn, Vec::new().into()));
                }
                if len < 0 || message.m_pData.is_null() {
                    return Err(Error::from_reason("Invalid Steam message payload"));
                }
                Ok(wrap(
                    message.m_conn,
                    slice::from_raw_parts(message.m_pData.cast::<u8>(), len as usize)
                        .to_vec()
                        .into(),
                ))
            })
            .collect()
    }

    #[napi(strict)]
    pub fn receive_messages(connection: f64, max_messages: Option<f64>) -> Result<Vec<Buffer>> {
        receive(
            handle(connection)?,
            max_messages,
            sys::SteamAPI_ISteamNetworkingSockets_ReceiveMessagesOnConnection,
            |_, data| data,
        )
    }

    #[napi(object)]
    pub struct PollGroupMessage {
        pub connection: u32,
        pub data: Buffer,
    }

    #[napi]
    pub fn create_poll_group() -> Result<u32> {
        created(
            unsafe { sys::SteamAPI_ISteamNetworkingSockets_CreatePollGroup(sockets()?) },
            "CreatePollGroup",
        )
    }

    #[napi(strict)]
    pub fn set_connection_poll_group(connection: f64, group: f64) -> Result<bool> {
        let conn = handle(connection)?;
        let group = integer(group, 0, u32::MAX, "group")?;
        Ok(unsafe {
            sys::SteamAPI_ISteamNetworkingSockets_SetConnectionPollGroup(sockets()?, conn, group)
        })
    }

    #[napi(strict)]
    pub fn receive_messages_on_poll_group(
        group: f64,
        max_messages: Option<f64>,
    ) -> Result<Vec<PollGroupMessage>> {
        receive(
            handle(group)?,
            max_messages,
            sys::SteamAPI_ISteamNetworkingSockets_ReceiveMessagesOnPollGroup,
            |connection, data| PollGroupMessage { connection, data },
        )
    }

    #[napi(strict)]
    pub fn destroy_poll_group(group: f64) -> Result<bool> {
        let group = handle(group)?;
        Ok(unsafe { sys::SteamAPI_ISteamNetworkingSockets_DestroyPollGroup(sockets()?, group) })
    }

    #[napi(strict)]
    pub fn close_connection(connection: f64, linger: Option<bool>) -> Result<bool> {
        let conn = handle(connection)?;
        Ok(unsafe {
            sys::SteamAPI_ISteamNetworkingSockets_CloseConnection(
                sockets()?,
                conn,
                0,
                ptr::null(),
                linger.unwrap_or(false),
            )
        })
    }

    #[napi(strict)]
    pub fn close_listen_socket(socket: f64) -> Result<bool> {
        let socket = handle(socket)?;
        Ok(unsafe { sys::SteamAPI_ISteamNetworkingSockets_CloseListenSocket(sockets()?, socket) })
    }

    #[napi]
    pub fn init_relay_network_access() {
        crate::client::get_client()
            .networking_utils()
            .init_relay_network_access();
    }

    #[napi]
    pub fn relay_status() -> String {
        match crate::client::get_client()
            .networking_utils()
            .relay_network_status()
        {
            Ok(status) => format!("{status:?}"),
            Err(status) => format!("{status:?}"),
        }
    }
}
