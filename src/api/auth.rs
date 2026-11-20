use napi_derive::napi;

#[napi]
pub mod auth {
    use std::net::SocketAddr;

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
        pub(crate) handle: AuthTicket,
    }

    #[napi]
    impl Ticket {
        #[napi]
        pub fn cancel(&mut self) {
            let client = crate::client::get_client();
            client.user().cancel_authentication_ticket(self.handle);
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
            NetworkingIdentity::new_steam_id(SteamId::from_raw(steam_id64.get_u64().1)),
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
        let (tx, rx) = oneshot::channel();
        let mut tx = Some(tx);

        let (ticket_handle, ticket) = client
            .user()
            .authentication_session_ticket(network_identity);

        let callback =
            client.register_callback(move |session_ticket_response: AuthSessionTicketResponse| {
                if session_ticket_response.ticket == ticket_handle {
                    if let Some(tx) = tx.take() {
                        tx.send(match session_ticket_response.result {
                            Ok(()) => Ok(()),
                            Err(e) => Err(Error::from_reason(e.to_string())),
                        })
                        .unwrap();
                    }
                }
            });

        let mut ticket = Ticket {
            data: ticket,
            handle: ticket_handle,
        };

        let timeout_seconds = u64::from(timeout_seconds.unwrap_or(10));
        let result =
            tokio::time::timeout(std::time::Duration::from_secs(timeout_seconds), rx).await;

        drop(callback);

        match result {
            Ok(result) => match result {
                Ok(Ok(())) => Ok(ticket),
                Ok(Err(e)) => {
                    ticket.cancel();
                    Err(e)
                }
                Err(e) => {
                    ticket.cancel();
                    Err(Error::from_reason(e.to_string()))
                }
            },
            Err(_) => {
                ticket.cancel();
                Err(Error::from_reason(
                    "Steam didn't validated the ticket in time.",
                ))
            }
        }
    }

    #[napi]
    pub async fn get_auth_ticket_for_web_api(
        identity: String,
        timeout_seconds: Option<u32>,
    ) -> Result<Ticket, Error> {
        let client = crate::client::get_client();
        let (tx, rx) = oneshot::channel();
        let mut tx = Some(tx);

        let ticket_handle = client
            .user()
            .authentication_session_ticket_for_webapi(&identity);

        let callback =
            client.register_callback(move |ticket_for_webapi_response: TicketForWebApiResponse| {
                if ticket_for_webapi_response.ticket_handle == ticket_handle {
                    let mut ticket = ticket_for_webapi_response.ticket;
                    ticket.truncate(ticket_for_webapi_response.ticket_len as usize);

                    if let Some(tx) = tx.take() {
                        tx.send(match ticket_for_webapi_response.result {
                            Ok(()) => Ok(ticket),
                            Err(e) => Err(Error::from_reason(e.to_string())),
                        })
                        .unwrap();
                    }
                }
            });

        let timeout_seconds = u64::from(timeout_seconds.unwrap_or(10));
        let result =
            tokio::time::timeout(std::time::Duration::from_secs(timeout_seconds), rx).await;

        drop(callback);

        match result {
            Ok(result) => match result {
                Ok(Ok(data)) => Ok(Ticket {
                    handle: ticket_handle,
                    data,
                }),
                Ok(Err(e)) => {
                    client.user().cancel_authentication_ticket(ticket_handle);
                    Err(e)
                }
                Err(e) => {
                    client.user().cancel_authentication_ticket(ticket_handle);
                    Err(Error::from_reason(e.to_string()))
                }
            },
            Err(_) => {
                client.user().cancel_authentication_ticket(ticket_handle);
                Err(Error::from_reason(
                    "Steam didn't validated the ticket in time.",
                ))
            }
        }
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
