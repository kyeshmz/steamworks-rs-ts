use napi_derive::napi;

#[napi]
pub mod networking {
    use napi::{
        bindgen_prelude::{BigInt, Buffer},
        Error, Status,
    };
    use steamworks::{sys, SteamId};

    use crate::api::localplayer::PlayerSteamId;

    #[napi(object)]
    pub struct P2PPacket {
        pub data: Buffer,
        pub size: i32,
        pub steam_id: PlayerSteamId,
    }

    #[napi]
    /// The method used to send a packet
    pub enum SendType {
        /// Send the packet directly over udp.
        ///
        /// Can't be larger than 1200 bytes
        Unreliable,
        /// Like `Unreliable` but doesn't buffer packets
        /// sent before the connection has started.
        UnreliableNoDelay,
        /// Reliable packet sending.
        ///
        /// Can't be larger than 1 megabyte.
        Reliable,
        /// Like `Reliable` but applies the nagle
        /// algorithm to packets being sent
        ReliableWithBuffering,
    }

    const MAX_PACKET: u32 = 1_048_576;

    fn integer(name: &str, value: f64, max: u32) -> Result<u32, Error> {
        if value.fract() == 0.0 && (0.0..=f64::from(max)).contains(&value) {
            Ok(value as u32)
        } else {
            Err(Error::new(
                Status::InvalidArg,
                format!("{name} must be an integer from 0 to {max}"),
            ))
        }
    }

    fn peer(id: BigInt) -> Result<u64, Error> {
        let (_, raw, lossless) = id.get_u64();
        let account_type = (raw >> 52) & 0xf;
        let instance = (raw >> 32) & 0xfffff;
        if lossless
            && (1..=4).contains(&(raw >> 56))
            && raw as u32 != 0
            && ((account_type == 1 && instance == 1) || matches!(account_type, 3 | 4))
        {
            Ok(raw)
        } else {
            Err(Error::new(Status::InvalidArg, "Expected a peer SteamID64"))
        }
    }

    fn interface() -> Result<*mut sys::ISteamNetworking, Error> {
        let networking = unsafe { sys::SteamAPI_SteamNetworking_v006() };
        if networking.is_null() {
            Err(Error::from_reason("Steam Networking unavailable"))
        } else {
            Ok(networking)
        }
    }

    #[napi(strict)]
    pub fn send_p2p_packet(
        steam_id64: BigInt,
        send_type: SendType,
        data: Buffer,
    ) -> Result<bool, Error> {
        send_p2p_packet_on_channel(steam_id64, send_type, data, 0.0)
    }

    #[napi(strict)]
    pub fn send_p2p_packet_on_channel(
        steam_id64: BigInt,
        send_type: SendType,
        data: Buffer,
        channel: f64,
    ) -> Result<bool, Error> {
        use sys::EP2PSend::*;
        let peer = peer(steam_id64)?;
        let channel = integer("channel", channel, i32::MAX as u32)? as i32;
        let (mode, limit) = match send_type {
            SendType::Unreliable => (k_EP2PSendUnreliable, 1200),
            SendType::UnreliableNoDelay => (k_EP2PSendUnreliableNoDelay, 1200),
            SendType::Reliable => (k_EP2PSendReliable, MAX_PACKET),
            SendType::ReliableWithBuffering => (k_EP2PSendReliableWithBuffering, MAX_PACKET),
        };
        if data.len() > limit as usize {
            return Err(Error::new(
                Status::InvalidArg,
                format!("Packet exceeds {limit} bytes"),
            ));
        }
        Ok(unsafe {
            sys::SteamAPI_ISteamNetworking_SendP2PPacket(
                interface()?,
                peer,
                data.as_ptr().cast(),
                data.len() as u32,
                mode,
                channel,
            )
        })
    }

    #[napi]
    pub fn is_p2p_packet_available() -> Result<u32, Error> {
        is_p2p_packet_available_on_channel(0.0)
    }

    #[napi(strict)]
    pub fn is_p2p_packet_available_on_channel(channel: f64) -> Result<u32, Error> {
        let channel = integer("channel", channel, i32::MAX as u32)? as i32;
        let mut size = 0;
        if unsafe {
            sys::SteamAPI_ISteamNetworking_IsP2PPacketAvailable(interface()?, &mut size, channel)
        } {
            Ok(size)
        } else {
            Ok(0)
        }
    }

    #[napi(strict)]
    pub fn read_p2p_packet(size: f64) -> Result<P2PPacket, Error> {
        read_p2p_packet_from_channel(size, 0.0)
    }

    #[napi(strict)]
    pub fn read_p2p_packet_from_channel(size: f64, channel: f64) -> Result<P2PPacket, Error> {
        let size = integer("packet size", size, MAX_PACKET)?;
        let channel = integer("channel", channel, i32::MAX as u32)? as i32;
        let networking = interface()?;
        let mut data = vec![0u8; size as usize];
        let mut read = 0;
        let mut remote = sys::CSteamID {
            m_steamid: sys::CSteamID_SteamID_t { m_unAll64Bits: 0 },
        };
        if !unsafe {
            sys::SteamAPI_ISteamNetworking_ReadP2PPacket(
                networking,
                data.as_mut_ptr().cast(),
                size,
                &mut read,
                &mut remote,
                channel,
            )
        } {
            return Err(Error::from_reason("No packet available"));
        }
        if read > size {
            return Err(Error::from_reason(
                "Packet exceeded the supplied buffer size",
            ));
        }
        data.truncate(read as usize);
        Ok(P2PPacket {
            data: data.into(),
            size: read as i32,
            steam_id: PlayerSteamId::from_steamid(SteamId::from_raw(unsafe {
                remote.m_steamid.m_unAll64Bits
            })),
        })
    }

    #[napi(strict)]
    pub fn accept_p2p_session(steam_id64: BigInt) -> Result<(), Error> {
        let peer = peer(steam_id64)?;
        unsafe { sys::SteamAPI_ISteamNetworking_AcceptP2PSessionWithUser(interface()?, peer) };
        Ok(())
    }

    #[napi(strict)]
    pub fn close_p2p_session(steam_id64: BigInt) -> Result<(), Error> {
        let peer = peer(steam_id64)?;
        unsafe { sys::SteamAPI_ISteamNetworking_CloseP2PSessionWithUser(interface()?, peer) };
        Ok(())
    }
}
