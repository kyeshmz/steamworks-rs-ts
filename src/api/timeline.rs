use napi_derive::napi;

#[napi]
pub mod timeline {
    use napi::bindgen_prelude::Error;
    use std::ffi::CString;
    use std::time::Duration;

    #[napi]
    pub enum GameMode {
        Playing,
        Staging,
        Menus,
        LoadingScreen,
    }

    #[napi]
    pub enum ClipPriority {
        None,
        Standard,
        Featured,
    }

    fn duration(seconds: f64) -> Result<Duration, Error> {
        if !seconds.is_finite() || seconds < 0.0 || seconds > f32::MAX as f64 {
            return Err(Error::from_reason(
                "Duration must be finite and non-negative",
            ));
        }
        Duration::try_from_secs_f64(seconds)
            .map_err(|_| Error::from_reason("Duration is out of range"))
    }

    #[napi]
    pub fn set_timeline_game_mode(mode: GameMode) {
        let mode = match mode {
            GameMode::Playing => steamworks::TimelineGameMode::Playing,
            GameMode::Staging => steamworks::TimelineGameMode::Staging,
            GameMode::Menus => steamworks::TimelineGameMode::Menus,
            GameMode::LoadingScreen => steamworks::TimelineGameMode::LoadingScreen,
        };
        crate::client::get_client()
            .timeline()
            .set_timeline_game_mode(mode);
    }

    #[napi]
    pub fn set_timeline_state_description(description: String, seconds: f64) -> Result<(), Error> {
        if description.contains('\0') {
            return Err(Error::from_reason("Description contains a NUL byte"));
        }
        let duration = duration(seconds)?;
        crate::client::get_client()
            .timeline()
            .set_timeline_state_description(&description, duration);
        Ok(())
    }

    #[napi]
    pub fn clear_timeline_state_description(seconds: f64) -> Result<(), Error> {
        let duration = duration(seconds)?;
        crate::client::get_client()
            .timeline()
            .clear_timeline_state_description(duration);
        Ok(())
    }

    #[napi]
    pub fn add_timeline_event(
        icon: String,
        title: String,
        description: String,
        priority: f64,
        start_offset_seconds: f64,
        duration_seconds: f64,
        clip_priority: ClipPriority,
    ) -> Result<(), Error> {
        if [&icon, &title, &description]
            .iter()
            .any(|s| s.contains('\0'))
        {
            return Err(Error::from_reason("Timeline text contains a NUL byte"));
        }
        if !start_offset_seconds.is_finite() || start_offset_seconds.abs() > f32::MAX as f64 {
            return Err(Error::from_reason(
                "Start offset must be a finite 32-bit float",
            ));
        }
        if !priority.is_finite()
            || priority.fract() != 0.0
            || !(0.0..=f64::from(steamworks::sys::k_unMaxTimelinePriority)).contains(&priority)
        {
            return Err(Error::from_reason(
                "Timeline priority must be an integer from 0 to 1000",
            ));
        }
        if duration_seconds > f64::from(steamworks::sys::k_flMaxTimelineEventDuration) {
            return Err(Error::from_reason(
                "Timeline event duration must not exceed 600 seconds",
            ));
        }
        let duration = duration(duration_seconds)?;
        let clip_priority = match clip_priority {
            ClipPriority::None => {
                steamworks::sys::ETimelineEventClipPriority::k_ETimelineEventClipPriority_None
            }
            ClipPriority::Standard => {
                steamworks::sys::ETimelineEventClipPriority::k_ETimelineEventClipPriority_Standard
            }
            ClipPriority::Featured => {
                steamworks::sys::ETimelineEventClipPriority::k_ETimelineEventClipPriority_Featured
            }
        };
        let icon = CString::new(icon)
            .map_err(|_| Error::from_reason("Timeline icon contains a NUL byte"))?;
        let title = CString::new(title)
            .map_err(|_| Error::from_reason("Timeline title contains a NUL byte"))?;
        let description = CString::new(description)
            .map_err(|_| Error::from_reason("Timeline description contains a NUL byte"))?;
        let _client = crate::client::get_client();
        unsafe {
            let timeline = steamworks::sys::SteamAPI_SteamTimeline_v004();
            if !timeline.is_null() {
                steamworks::sys::SteamAPI_ISteamTimeline_AddRangeTimelineEvent(
                    timeline,
                    title.as_ptr(),
                    description.as_ptr(),
                    icon.as_ptr(),
                    priority as u32,
                    start_offset_seconds as f32,
                    duration.as_secs_f32(),
                    clip_priority,
                );
            }
        }
        Ok(())
    }
}
