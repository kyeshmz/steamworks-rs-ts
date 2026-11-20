use napi_derive::napi;

#[napi]
pub mod auth {
    use std::net::SocketAddr;
    use std::sync::{Mutex as StdMutex, OnceLock};

    use napi::bindgen_prelude::{BigInt, Buffer, Error};
    use napi::Env;
    use steamworks::sys;
    use steamworks::{
        networking_types::NetworkingIdentity, AuthSessionTicketResponse, AuthTicket, SteamId,
        TicketForWebApiResponse,
    };
    use tokio::sync::{oneshot, Mutex};

    const MAX_APP_DATA: usize = 128;
    const TICKET_BUFFER: usize = 1024;

    lazy_static! {
        static ref ENCRYPTED_TICKET_SLOT: Mutex<()> = Mutex::new(());
        static ref SESSION_REQUESTS: StdMutex<Pending<Result<(), String>>> =
            StdMutex::new(Pending::default());
        static ref WEB_REQUESTS: StdMutex<Pending<Result<Vec<u8>, String>>> =
            StdMutex::new(Pending::default());
    }

    struct Pending<V>(Vec<(AuthTicket, oneshot::Sender<V>)>);

    impl<V> Default for Pending<V> {
        fn default() -> Self {
            Self(Vec::new())
        }
    }

    impl<V> Pending<V> {
        fn insert(&mut self, handle: AuthTicket, sender: oneshot::Sender<V>) {
            self.0.push((handle, sender));
        }

        fn take(&mut self, handle: &AuthTicket) -> Option<oneshot::Sender<V>> {
            self.0
                .iter()
                .position(|(key, _)| key == handle)
                .map(|index| self.0.swap_remove(index).1)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::{get_session_ticket_with_steam_id, session_peer, web_ticket_result};
        use napi::bindgen_prelude::BigInt;
        use std::future::Future;
        use std::task::{Context, Poll, Waker};
        use steamworks::{sys, Callback, TicketForWebApiResponse};

        fn response(length: i32, result: sys::EResult) -> TicketForWebApiResponse {
            let mut raw = sys::GetTicketForWebApiResponse_t {
                m_hAuthTicket: 1,
                m_eResult: result,
                m_cubTicket: length,
                m_rgubTicket: [7; 2560],
            };
            unsafe { TicketForWebApiResponse::from_raw(&mut raw as *mut _ as *mut _) }
        }

        #[test]
        fn web_ticket_rejects_invalid_lengths_and_preserves_status_errors() {
            for length in [-1, 0, 2561] {
                assert!(web_ticket_result(response(length, sys::EResult::k_EResultOK)).is_err());
            }
            assert_eq!(
                web_ticket_result(response(3, sys::EResult::k_EResultOK)).unwrap(),
                vec![7, 7, 7]
            );
            assert_eq!(
                web_ticket_result(response(2560, sys::EResult::k_EResultOK)).unwrap(),
                vec![7; 2560]
            );
            let error = web_ticket_result(response(-1, sys::EResult::k_EResultFail)).unwrap_err();
            assert_eq!(error, "a generic failure from the steamworks API");
        }

        #[test]
        fn invalid_session_peers_fail_before_request_submission() {
            for (sign_bit, words) in [(false, vec![0]), (true, vec![1]), (false, vec![1, 1])] {
                let mut request = Box::pin(get_session_ticket_with_steam_id(
                    BigInt { sign_bit, words },
                    None,
                ));
                let mut cx = Context::from_waker(Waker::noop());
                assert!(matches!(
                    request.as_mut().poll(&mut cx),
                    Poll::Ready(Err(_))
                ));
            }
            for raw in [1, u64::MAX] {
                assert_eq!(
                    session_peer(BigInt {
                        sign_bit: false,
                        words: vec![raw],
                    })
                    .unwrap()
                    .raw(),
                    raw
                );
            }
        }
    }

    fn register_ticket_callbacks(client: &steamworks::Client) {
        static CALLBACKS: OnceLock<(steamworks::CallbackHandle, steamworks::CallbackHandle)> =
            OnceLock::new();
        CALLBACKS.get_or_init(|| {
            let session = client.register_callback(|response: AuthSessionTicketResponse| {
                let sender = SESSION_REQUESTS.lock().unwrap().take(&response.ticket);
                if let Some(sender) = sender {
                    let _ = sender.send(response.result.map_err(|error| error.to_string()));
                }
            });
            let web = client.register_callback(|response: TicketForWebApiResponse| {
                let sender = WEB_REQUESTS.lock().unwrap().take(&response.ticket_handle);
                if let Some(sender) = sender {
                    let _ = sender.send(web_ticket_result(response));
                }
            });
            (session, web)
        });
    }

    fn web_ticket_result(response: TicketForWebApiResponse) -> Result<Vec<u8>, String> {
        response.result.map_err(|error| error.to_string())?;
        let length = usize::try_from(response.ticket_len)
            .ok()
            .filter(|length| *length > 0 && *length <= response.ticket.len())
            .ok_or_else(|| "Steam returned an invalid web API ticket length".to_string())?;
        let mut ticket = response.ticket;
        ticket.truncate(length);
        Ok(ticket)
    }

    async fn wait_for_ticket<T>(
        rx: oneshot::Receiver<Result<T, String>>,
        timeout_seconds: Option<u32>,
    ) -> Result<T, Error> {
        tokio::time::timeout(
            std::time::Duration::from_secs(u64::from(timeout_seconds.unwrap_or(10))),
            rx,
        )
        .await
        .map_err(|_| Error::from_reason("Steam didn't validated the ticket in time."))?
        .map_err(|error| Error::from_reason(error.to_string()))?
        .map_err(Error::from_reason)
    }

    enum TicketKind {
        Session,
        Web,
    }

    struct PendingTicket {
        handle: AuthTicket,
        kind: TicketKind,
        completed: bool,
    }

    impl Drop for PendingTicket {
        fn drop(&mut self) {
            if self.completed {
                return;
            }
            match self.kind {
                TicketKind::Session => {
                    SESSION_REQUESTS.lock().unwrap().take(&self.handle);
                }
                TicketKind::Web => {
                    WEB_REQUESTS.lock().unwrap().take(&self.handle);
                }
            }
            crate::client::get_client()
                .user()
                .cancel_authentication_ticket(self.handle);
        }
    }

    fn steam_user() -> Result<*mut sys::ISteamUser, Error> {
        let user = unsafe { sys::SteamAPI_SteamUser_v023() };
        if user.is_null() {
            Err(Error::from_reason("Steam User unavailable"))
        } else {
            Ok(user)
        }
    }

    fn steam_utils() -> Result<*mut sys::ISteamUtils, Error> {
        let utils = unsafe { sys::SteamAPI_SteamUtils_v010() };
        if utils.is_null() {
            Err(Error::from_reason("Steam Utils unavailable"))
        } else {
            Ok(utils)
        }
    }

    fn session_peer(id: BigInt) -> Result<SteamId, Error> {
        let (_, raw, lossless) = id.get_u64();
        if !lossless || raw == 0 {
            return Err(Error::from_reason(
                "Expected a nonzero unsigned 64-bit Steam ID",
            ));
        }
        Ok(SteamId::from_raw(raw))
    }

    #[napi(strict)]
    pub fn begin_auth_session(steam_id64: BigInt, ticket: Buffer) -> Result<(), Error> {
        let peer = session_peer(steam_id64)?;
        if ticket.is_empty() || ticket.len() > i32::MAX as usize {
            return Err(Error::from_reason(
                "Expected a nonempty ticket fitting a signed 32-bit length",
            ));
        }
        crate::client::get_client()
            .user()
            .begin_authentication_session(peer, &ticket)
            .map_err(|error| Error::from_reason(error.to_string()))
    }

    #[napi(strict)]
    pub fn end_auth_session(steam_id64: BigInt) -> Result<(), Error> {
        let peer = session_peer(steam_id64)?;
        crate::client::get_client()
            .user()
            .end_authentication_session(peer);
        Ok(())
    }

    #[napi]
    pub struct Ticket {
        pub(crate) data: Vec<u8>,
        pub(crate) handle: Option<AuthTicket>,
    }

    #[napi]
    impl Ticket {
        #[napi]
        pub fn cancel(&mut self) {
            if let Some(handle) = self.handle.take() {
                crate::client::get_client()
                    .user()
                    .cancel_authentication_ticket(handle);
            }
        }

        #[napi]
        pub fn get_bytes(&self) -> Buffer {
            self.data.clone().into()
        }
    }

    #[napi]
    pub async fn get_session_ticket_with_steam_id(
        steam_id64: BigInt,
        timeout_seconds: Option<u32>,
    ) -> Result<Ticket, Error> {
        get_session_ticket(
            NetworkingIdentity::new_steam_id(session_peer(steam_id64)?),
            timeout_seconds,
        )
        .await
    }

    #[napi]
    pub async fn get_session_ticket_with_ip(
        ip: String,
        timeout_seconds: Option<u32>,
    ) -> Result<Ticket, Error> {
        match ip.parse::<SocketAddr>() {
            Ok(addr) => get_session_ticket(NetworkingIdentity::new_ip(addr), timeout_seconds).await,
            Err(e) => Err(Error::from_reason(e.to_string())),
        }
    }

    async fn get_session_ticket(
        network_identity: NetworkingIdentity,
        timeout_seconds: Option<u32>,
    ) -> Result<Ticket, Error> {
        let client = crate::client::get_client();
        register_ticket_callbacks(&client);
        let (tx, rx) = oneshot::channel();
        let (ticket_handle, ticket) = crate::api::runtime::without_dispatch(|| {
            let (handle, ticket) = client
                .user()
                .authentication_session_ticket(network_identity);
            SESSION_REQUESTS.lock().unwrap().insert(handle, tx);
            (handle, ticket)
        });
        let mut pending = PendingTicket {
            handle: ticket_handle,
            kind: TicketKind::Session,
            completed: false,
        };
        wait_for_ticket(rx, timeout_seconds).await?;
        pending.completed = true;
        Ok(Ticket {
            data: ticket,
            handle: Some(ticket_handle),
        })
    }

    #[napi]
    pub async fn get_auth_ticket_for_web_api(
        identity: String,
        timeout_seconds: Option<u32>,
    ) -> Result<Ticket, Error> {
        if identity.as_bytes().contains(&0) {
            return Err(Error::from_reason("Identity cannot contain a NUL byte"));
        }
        let client = crate::client::get_client();
        register_ticket_callbacks(&client);
        let (tx, rx) = oneshot::channel();
        let ticket_handle = crate::api::runtime::without_dispatch(|| {
            let handle = client
                .user()
                .authentication_session_ticket_for_webapi(&identity);
            WEB_REQUESTS.lock().unwrap().insert(handle, tx);
            handle
        });
        let mut pending = PendingTicket {
            handle: ticket_handle,
            kind: TicketKind::Web,
            completed: false,
        };
        let data = wait_for_ticket(rx, timeout_seconds).await?;
        pending.completed = true;
        Ok(Ticket {
            handle: Some(ticket_handle),
            data,
        })
    }

    fn encrypted_ticket_response(
        utils: *mut sys::ISteamUtils,
        call: sys::SteamAPICall_t,
    ) -> Result<sys::EncryptedAppTicketResponse_t, Error> {
        let mut response: sys::EncryptedAppTicketResponse_t = unsafe { std::mem::zeroed() };
        let mut failed = false;
        let fetched = unsafe {
            sys::SteamAPI_ISteamUtils_GetAPICallResult(
                utils,
                call,
                (&mut response) as *mut _ as *mut _,
                std::mem::size_of::<sys::EncryptedAppTicketResponse_t>() as i32,
                sys::EncryptedAppTicketResponse_t_k_iCallback as i32,
                &mut failed,
            )
        };
        if !fetched || failed {
            let reason = unsafe { sys::SteamAPI_ISteamUtils_GetAPICallFailureReason(utils, call) };
            return Err(Error::from_reason(format!(
                "Steam failed the encrypted app ticket request: {reason:?}"
            )));
        }
        Ok(response)
    }

    fn encrypted_ticket_bytes(user: *mut sys::ISteamUser) -> Result<Buffer, Error> {
        let mut buffer = vec![0u8; TICKET_BUFFER];
        let mut ticket_len: u32 = 0;
        let mut copied = unsafe {
            sys::SteamAPI_ISteamUser_GetEncryptedAppTicket(
                user,
                buffer.as_mut_ptr() as *mut _,
                buffer.len() as i32,
                &mut ticket_len,
            )
        };
        if !copied && (buffer.len() + 1..=64 * 1024).contains(&(ticket_len as usize)) {
            buffer.resize(ticket_len as usize, 0);
            copied = unsafe {
                sys::SteamAPI_ISteamUser_GetEncryptedAppTicket(
                    user,
                    buffer.as_mut_ptr() as *mut _,
                    buffer.len() as i32,
                    &mut ticket_len,
                )
            };
        }
        if !copied || ticket_len == 0 || ticket_len as usize > buffer.len() {
            return Err(Error::from_reason(
                "Steam completed the request without an encrypted app ticket",
            ));
        }
        buffer.truncate(ticket_len as usize);
        Ok(buffer.into())
    }

    #[napi(ts_return_type = "Promise<Buffer>")]
    pub fn request_encrypted_app_ticket(
        env: &Env,
        data_to_include: Option<Buffer>,
        timeout_seconds: Option<f64>,
    ) -> Result<napi::bindgen_prelude::PromiseRaw<'_, Buffer>, Error> {
        let bytes = data_to_include.as_deref().unwrap_or_default();
        let data = (bytes.len() <= MAX_APP_DATA).then(|| bytes.to_vec());
        let seconds = timeout_seconds.unwrap_or(10.0);
        env.spawn_future(async move {
            let data = data.ok_or_else(|| {
                Error::from_reason("Expected at most 128 bytes of application data")
            })?;
            if seconds.fract() != 0.0 || !(1.0..=u32::MAX as f64).contains(&seconds) {
                return Err(Error::from_reason(
                    "Expected a positive integer timeout in seconds",
                ));
            }
            tokio::time::timeout(
                std::time::Duration::from_secs(seconds as u64),
                request_ticket(data),
            )
            .await
            .map_err(|_| Error::from_reason("Steam encrypted app ticket request timed out"))?
        })
    }

    async fn request_ticket(data: Vec<u8>) -> Result<Buffer, Error> {
        let _slot = ENCRYPTED_TICKET_SLOT.lock().await;
        let call = unsafe {
            sys::SteamAPI_ISteamUser_RequestEncryptedAppTicket(
                steam_user()?,
                if data.is_empty() {
                    std::ptr::null_mut()
                } else {
                    data.as_ptr() as *mut _
                },
                data.len() as i32,
            )
        };
        if call == sys::k_uAPICallInvalid {
            return Err(Error::from_reason(
                "Steam did not accept the encrypted app ticket request",
            ));
        }

        let mut poll = tokio::time::interval(std::time::Duration::from_millis(100));
        loop {
            poll.tick().await;

            let utils = steam_utils()?;
            let mut failed = false;
            let completed =
                unsafe { sys::SteamAPI_ISteamUtils_IsAPICallCompleted(utils, call, &mut failed) };
            if failed {
                return Err(Error::from_reason(
                    "Steam failed the encrypted app ticket call",
                ));
            }
            if !completed {
                continue;
            }

            let response = encrypted_ticket_response(utils, call)?;
            return match response.m_eResult {
                sys::EResult::k_EResultOK => encrypted_ticket_bytes(steam_user()?),
                result => Err(Error::from_reason(format!(
                    "Steam rejected the encrypted app ticket request: {result:?}"
                ))),
            };
        }
    }
}
