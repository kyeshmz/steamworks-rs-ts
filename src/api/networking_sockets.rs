use napi_derive::napi;

#[napi]
pub mod networking_sockets {
    use napi::{
        bindgen_prelude::{BigInt, Buffer, Function},
        threadsafe_function::ThreadsafeFunctionCallMode,
        Error, Result,
    };
    use std::{ffi::CString, mem::MaybeUninit, net::IpAddr, ptr, slice, sync::Mutex};
    use steamworks::{networking_types::NetworkingIdentity, sys, Callback, CallbackHandle};

    static STATUS_HANDLER: Mutex<Option<CallbackHandle>> = Mutex::new(None);

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

        unsafe fn from_raw(raw: *mut std::ffi::c_void) -> Self {
            let event = raw
                .cast::<sys::SteamNetConnectionStatusChangedCallback_t>()
                .read_unaligned();
            info(event.m_hConn, event.m_info)
        }
    }

    #[napi(strict)]
    pub fn set_connection_status_handler(
        #[napi(ts_arg_type = "(info: ConnectionInfo) => void")] handler: Option<
            Function<'_, ConnectionInfo, ()>,
        >,
    ) -> Result<()> {
        let callback = handler
            .map(|handler| handler.build_threadsafe_function().weak::<true>().build())
            .transpose()?;
        let mut slot = STATUS_HANDLER
            .lock()
            .map_err(|_| Error::from_reason("Connection handler lock poisoned"))?;
        drop(slot.take());
        if let Some(callback) = callback {
            *slot = Some(crate::client::get_client().register_callback(
                move |event: ConnectionInfo| {
                    callback.call(event, ThreadsafeFunctionCallMode::NonBlocking);
                },
            ));
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

    #[napi(strict)]
    pub fn connection_user_data(connection: f64) -> Result<Option<BigInt>> {
        let connection = handle(connection)?;
        let value = unsafe {
            sys::SteamAPI_ISteamNetworkingSockets_GetConnectionUserData(sockets()?, connection)
        };
        Ok((value != -1).then(|| value.into()))
    }

    #[napi(strict)]
    pub fn set_connection_user_data(connection: f64, value: BigInt) -> Result<bool> {
        let connection = handle(connection)?;
        let (value, exact) = value.get_i64();
        if !exact {
            return Err(Error::from_reason(
                "userData must fit in a signed 64-bit integer",
            ));
        }
        Ok(unsafe {
            sys::SteamAPI_ISteamNetworkingSockets_SetConnectionUserData(
                sockets()?,
                connection,
                value,
            )
        })
    }

    #[napi(strict)]
    pub fn connection_name(connection: f64) -> Result<Option<String>> {
        let connection = handle(connection)?;
        let mut bytes = [0u8; 1024];
        let ok = unsafe {
            sys::SteamAPI_ISteamNetworkingSockets_GetConnectionName(
                sockets()?,
                connection,
                bytes.as_mut_ptr().cast(),
                bytes.len() as i32,
            )
        };
        Ok(ok.then(|| {
            let length = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
            String::from_utf8_lossy(&bytes[..length]).into_owned()
        }))
    }

    #[napi(strict)]
    pub fn set_connection_name(connection: f64, name: String) -> Result<()> {
        let connection = handle(connection)?;
        let name = CString::new(name).map_err(|_| Error::from_reason("Name contains NUL"))?;
        unsafe {
            sys::SteamAPI_ISteamNetworkingSockets_SetConnectionName(
                sockets()?,
                connection,
                name.as_ptr(),
            )
        };
        Ok(())
    }

    #[napi(strict)]
    pub fn flush_messages(connection: f64) -> Result<()> {
        let connection = handle(connection)?;
        check(
            unsafe {
                sys::SteamAPI_ISteamNetworkingSockets_FlushMessagesOnConnection(
                    sockets()?,
                    connection,
                )
            },
            "FlushMessagesOnConnection",
        )
    }

    #[napi]
    pub fn init_authentication() -> Result<i32> {
        Ok(unsafe { sys::SteamAPI_ISteamNetworkingSockets_InitAuthentication(sockets()?) } as i32)
    }

    #[napi(object)]
    pub struct AuthenticationStatus {
        pub availability: i32,
        pub debug_message: String,
    }

    #[napi]
    pub fn authentication_status() -> Result<AuthenticationStatus> {
        let mut details: sys::SteamNetAuthenticationStatus_t = unsafe { std::mem::zeroed() };
        let status = unsafe {
            sys::SteamAPI_ISteamNetworkingSockets_GetAuthenticationStatus(sockets()?, &mut details)
        };
        let bytes: Vec<u8> = details
            .m_debugMsg
            .iter()
            .take_while(|&&b| b != 0)
            .map(|&b| b as u8)
            .collect();
        Ok(AuthenticationStatus {
            availability: status as i32,
            debug_message: String::from_utf8_lossy(&bytes).into_owned(),
        })
    }

    #[napi(object)]
    pub struct LaneStatus {
        pub pending_unreliable: i32,
        pub pending_reliable: i32,
        pub sent_unacked_reliable: i32,
        pub queue_time_microseconds: BigInt,
    }

    #[napi(object)]
    pub struct RealtimeStatus {
        pub state: i32,
        pub ping: i32,
        pub quality_local: f64,
        pub quality_remote: f64,
        pub out_packets_per_second: f64,
        pub out_bytes_per_second: f64,
        pub in_packets_per_second: f64,
        pub in_bytes_per_second: f64,
        pub send_rate_bytes_per_second: i32,
        pub pending_unreliable: i32,
        pub pending_reliable: i32,
        pub sent_unacked_reliable: i32,
        pub queue_time_microseconds: BigInt,
        pub lanes: Vec<LaneStatus>,
    }

    #[napi(strict)]
    pub fn realtime_connection_status(
        connection: f64,
        lanes: Option<f64>,
    ) -> Result<RealtimeStatus> {
        let connection = handle(connection)?;
        let count = integer(lanes.unwrap_or(0.0), 0, 32, "lanes")? as usize;
        let mut status: sys::SteamNetConnectionRealTimeStatus_t = unsafe { std::mem::zeroed() };
        let mut lane_status: Vec<sys::SteamNetConnectionRealTimeLaneStatus_t> =
            (0..count).map(|_| unsafe { std::mem::zeroed() }).collect();
        check(
            unsafe {
                sys::SteamAPI_ISteamNetworkingSockets_GetConnectionRealTimeStatus(
                    sockets()?,
                    connection,
                    &mut status,
                    count as i32,
                    if count == 0 {
                        ptr::null_mut()
                    } else {
                        lane_status.as_mut_ptr()
                    },
                )
            },
            "GetConnectionRealTimeStatus",
        )?;
        Ok(RealtimeStatus {
            state: status.m_eState as i32,
            ping: status.m_nPing,
            quality_local: status.m_flConnectionQualityLocal as f64,
            quality_remote: status.m_flConnectionQualityRemote as f64,
            out_packets_per_second: status.m_flOutPacketsPerSec as f64,
            out_bytes_per_second: status.m_flOutBytesPerSec as f64,
            in_packets_per_second: status.m_flInPacketsPerSec as f64,
            in_bytes_per_second: status.m_flInBytesPerSec as f64,
            send_rate_bytes_per_second: status.m_nSendRateBytesPerSecond,
            pending_unreliable: status.m_cbPendingUnreliable,
            pending_reliable: status.m_cbPendingReliable,
            sent_unacked_reliable: status.m_cbSentUnackedReliable,
            queue_time_microseconds: status.m_usecQueueTime.into(),
            lanes: lane_status
                .into_iter()
                .map(|lane| LaneStatus {
                    pending_unreliable: lane.m_cbPendingUnreliable,
                    pending_reliable: lane.m_cbPendingReliable,
                    sent_unacked_reliable: lane.m_cbSentUnackedReliable,
                    queue_time_microseconds: lane.m_usecQueueTime.into(),
                })
                .collect(),
        })
    }

    #[napi(object)]
    pub struct LaneConfig {
        pub priority: f64,
        pub weight: f64,
    }

    #[napi(strict)]
    pub fn configure_connection_lanes(connection: f64, lanes: Vec<LaneConfig>) -> Result<()> {
        let connection = handle(connection)?;
        if lanes.is_empty() || lanes.len() > 32 {
            return Err(Error::from_reason("Expected 1 to 32 lanes"));
        }
        let mut priorities = Vec::with_capacity(lanes.len());
        let mut weights = Vec::with_capacity(lanes.len());
        for lane in lanes {
            if lane.priority.fract() != 0.0
                || !(i32::MIN as f64..=i32::MAX as f64).contains(&lane.priority)
            {
                return Err(Error::from_reason(
                    "priority must be a signed 32-bit integer",
                ));
            }
            priorities.push(lane.priority as i32);
            weights.push(integer(lane.weight, 1, u16::MAX as u32, "weight")? as u16);
        }
        check(
            unsafe {
                sys::SteamAPI_ISteamNetworkingSockets_ConfigureConnectionLanes(
                    sockets()?,
                    connection,
                    priorities.len() as i32,
                    priorities.as_ptr(),
                    weights.as_ptr(),
                )
            },
            "ConfigureConnectionLanes",
        )
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

    #[napi(object)]
    pub struct OutgoingMessage {
        pub connection: f64,
        pub data: Buffer,
        pub reliable: Option<bool>,
        pub lane: Option<f64>,
    }

    #[napi(object)]
    pub struct SendMessageResult {
        pub message_number: Option<BigInt>,
        pub error: Option<BigInt>,
    }

    #[napi(strict)]
    pub fn send_messages(messages: Vec<OutgoingMessage>) -> Result<Vec<SendMessageResult>> {
        if messages.len() > MAX_BATCH {
            return Err(Error::from_reason(
                "At most 32 messages may be sent at once",
            ));
        }
        let validated: Vec<_> = messages
            .iter()
            .map(|message| {
                let conn = handle(message.connection)?;
                let lane = integer(message.lane.unwrap_or(0.0), 0, u16::MAX as u32, "lane")?;
                if message.data.len() > 512 * 1024 {
                    return Err(Error::from_reason("Message exceeds 512 KiB"));
                }
                Ok((conn, lane))
            })
            .collect::<Result<_>>()?;
        if messages.is_empty() {
            return Ok(Vec::new());
        }
        let api = sockets()?;
        let utils = unsafe { sys::SteamAPI_SteamNetworkingUtils_SteamAPI_v004() };
        if utils.is_null() {
            return Err(Error::from_reason("Steam Networking Utils unavailable"));
        }
        struct OwnedMessages(Vec<*mut sys::SteamNetworkingMessage_t>);
        impl Drop for OwnedMessages {
            fn drop(&mut self) {
                for &message in &self.0 {
                    unsafe { sys::SteamAPI_SteamNetworkingMessage_t_Release(message) };
                }
            }
        }
        let mut allocated = OwnedMessages(Vec::with_capacity(messages.len()));
        for (message, (conn, lane)) in messages.iter().zip(validated) {
            let raw = unsafe {
                sys::SteamAPI_ISteamNetworkingUtils_AllocateMessage(
                    utils,
                    message.data.len() as i32,
                )
            };
            if raw.is_null() {
                return Err(Error::from_reason("Steam failed to allocate a message"));
            }
            allocated.0.push(raw);
            unsafe {
                if !message.data.is_empty() {
                    if (*raw).m_pData.is_null() {
                        return Err(Error::from_reason(
                            "Steam failed to allocate a message payload",
                        ));
                    }
                    ptr::copy_nonoverlapping(
                        message.data.as_ptr(),
                        (*raw).m_pData.cast(),
                        message.data.len(),
                    );
                }
                (*raw).m_conn = conn;
                (*raw).m_idxLane = lane as u16;
                (*raw).m_nFlags = if message.reliable.unwrap_or(true) {
                    sys::k_nSteamNetworkingSend_Reliable
                } else {
                    sys::k_nSteamNetworkingSend_Unreliable
                };
            }
        }
        let mut results = vec![0i64; messages.len()];
        unsafe {
            sys::SteamAPI_ISteamNetworkingSockets_SendMessages(
                api,
                allocated.0.len() as i32,
                allocated.0.as_ptr(),
                results.as_mut_ptr(),
            );
        }
        allocated.0.clear();
        Ok(results
            .into_iter()
            .map(|result| {
                if result >= 0 {
                    SendMessageResult {
                        message_number: Some(result.into()),
                        error: None,
                    }
                } else {
                    SendMessageResult {
                        message_number: None,
                        error: Some(result.unsigned_abs().into()),
                    }
                }
            })
            .collect())
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

    #[napi(object)]
    pub struct RelayNetworkStatus {
        pub availability: String,
        pub ping_measurement_in_progress: bool,
        pub network_config: String,
        pub any_relay: String,
        pub debug_message: String,
    }

    #[napi]
    pub fn detailed_relay_network_status() -> RelayNetworkStatus {
        let status = crate::client::get_client()
            .networking_utils()
            .detailed_relay_network_status();
        RelayNetworkStatus {
            availability: format!("{:?}", status.availability()),
            ping_measurement_in_progress: status.is_ping_measurement_in_progress(),
            network_config: format!("{:?}", status.network_config()),
            any_relay: format!("{:?}", status.any_relay()),
            debug_message: status.debugging_message().to_owned(),
        }
    }
}
