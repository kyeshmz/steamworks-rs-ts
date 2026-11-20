use napi_derive::napi;

#[napi]
pub mod voice {
    use napi::{bindgen_prelude::Buffer, Error, Result};
    use std::ptr;
    use steamworks::sys::{self, EVoiceResult, EVoiceResult::*};

    const MAX_COMPRESSED: usize = 64 * 1024;
    const MAX_PCM: usize = 1024 * 1024;

    fn user() -> Result<*mut sys::ISteamUser> {
        let user = unsafe { sys::SteamAPI_SteamUser_v023() };
        if user.is_null() {
            Err(Error::from_reason("Steam User unavailable"))
        } else {
            Ok(user)
        }
    }

    fn check_status(operation: &str, status: EVoiceResult) -> Result<()> {
        if status == k_EVoiceResultOK {
            Ok(())
        } else {
            Err(Error::from_reason(format!("{operation}: {status:?}")))
        }
    }

    fn output(mut bytes: Vec<u8>, written: u32) -> Result<Buffer> {
        if written as usize > bytes.len() {
            return Err(Error::from_reason("Steam returned an invalid voice length"));
        }
        bytes.truncate(written as usize);
        Ok(bytes.into())
    }

    #[napi]
    pub fn start_voice_recording() -> Result<()> {
        unsafe { sys::SteamAPI_ISteamUser_StartVoiceRecording(user()?) };
        Ok(())
    }

    #[napi]
    pub fn stop_voice_recording() -> Result<()> {
        unsafe { sys::SteamAPI_ISteamUser_StopVoiceRecording(user()?) };
        Ok(())
    }

    fn available(user: *mut sys::ISteamUser) -> Result<u32> {
        let mut available = 0;
        let status = unsafe {
            sys::SteamAPI_ISteamUser_GetAvailableVoice(user, &mut available, ptr::null_mut(), 0)
        };
        if status == k_EVoiceResultNoData {
            return Ok(0);
        }
        check_status("GetAvailableVoice", status)?;
        Ok(available)
    }

    #[napi]
    pub fn get_available_voice() -> Result<u32> {
        available(user()?)
    }

    #[napi]
    pub fn get_voice() -> Result<Buffer> {
        let user = user()?;
        let available = available(user)?;
        if available == 0 {
            return Ok(Vec::new().into());
        }
        if available as usize > MAX_COMPRESSED {
            return Err(Error::from_reason("Compressed voice exceeds 64 KiB"));
        }
        let mut bytes = vec![0; available as usize];
        let mut written = 0;
        let status = unsafe {
            sys::SteamAPI_ISteamUser_GetVoice(
                user,
                true,
                bytes.as_mut_ptr().cast(),
                available,
                &mut written,
                false,
                ptr::null_mut(),
                0,
                ptr::null_mut(),
                0,
            )
        };
        if status == k_EVoiceResultNoData {
            return Ok(Vec::new().into());
        }
        check_status("GetVoice", status)?;
        output(bytes, written)
    }

    #[napi(strict)]
    pub fn decompress_voice(compressed: Buffer, sample_rate: f64) -> Result<Buffer> {
        if !(1..=MAX_COMPRESSED).contains(&compressed.len()) {
            return Err(Error::from_reason(
                "compressed must contain 1 to 65536 bytes",
            ));
        }
        if sample_rate.fract() != 0.0 || !(11025.0..=48000.0).contains(&sample_rate) {
            return Err(Error::from_reason(
                "sampleRate must be an integer from 11025 to 48000",
            ));
        }
        let user = user()?;
        let decode = |dst: &mut [u8], written: &mut u32| unsafe {
            sys::SteamAPI_ISteamUser_DecompressVoice(
                user,
                compressed.as_ptr().cast(),
                compressed.len() as u32,
                dst.as_mut_ptr().cast(),
                dst.len() as u32,
                written,
                sample_rate as u32,
            )
        };
        let mut bytes = vec![0; 20 * 1024];
        let mut written = 0;
        let mut status = decode(&mut bytes, &mut written);
        if status == k_EVoiceResultBufferTooSmall {
            if !((bytes.len() + 1)..=MAX_PCM).contains(&(written as usize)) {
                return Err(Error::from_reason(
                    "Invalid required PCM size or PCM exceeds 1 MiB",
                ));
            }
            bytes.resize(written as usize, 0);
            written = 0;
            status = decode(&mut bytes, &mut written);
        }
        check_status("DecompressVoice", status)?;
        if written % 2 != 0 {
            return Err(Error::from_reason("Steam returned a partial PCM sample"));
        }
        output(bytes, written)
    }

    #[napi]
    pub fn get_voice_optimal_sample_rate() -> Result<u32> {
        Ok(unsafe { sys::SteamAPI_ISteamUser_GetVoiceOptimalSampleRate(user()?) })
    }
}
