use napi_derive::napi;

#[napi]
pub mod utils {
    use std::ffi::{CStr, CString};
    use steamworks::sys;
    use steamworks::FloatingGamepadTextInputMode as kFloatingGamepadTextInputMode;
    use steamworks::GamepadTextInputLineMode as kGamepadTextInputLineMode;
    use steamworks::GamepadTextInputMode as kGamepadTextInputMode;
    use tokio::sync::{oneshot, Mutex};

    static GAMEPAD_DIALOG: Mutex<()> = Mutex::const_new(());
    static FLOATING_DIALOG: Mutex<()> = Mutex::const_new(());

    #[napi]
    pub fn is_steam_in_big_picture_mode() -> napi::Result<bool> {
        let _client = crate::client::get_client();
        let utils = unsafe { sys::SteamAPI_SteamUtils_v010() };
        if utils.is_null() {
            return Err(napi::Error::from_reason("Steam utilities unavailable"));
        }
        Ok(unsafe { sys::SteamAPI_ISteamUtils_IsSteamInBigPictureMode(utils) })
    }

    #[napi(object, object_from_js = false)]
    pub struct SteamWarning {
        pub severity: i32,
        pub message: String,
    }

    #[napi(strict)]
    pub fn set_warning_handler(
        #[napi(ts_arg_type = "(warning: SteamWarning) => void")] handler: Option<
            napi::bindgen_prelude::Function<'_, SteamWarning, ()>,
        >,
    ) -> napi::Result<()> {
        let utils = crate::client::get_client().utils();
        if let Some(handler) = handler {
            let listener = handler.build_threadsafe_function().weak::<true>().build()?;
            utils.set_warning_callback(move |severity, message| {
                listener.call(
                    SteamWarning {
                        severity,
                        message: message.to_string_lossy().into_owned(),
                    },
                    napi::threadsafe_function::ThreadsafeFunctionCallMode::NonBlocking,
                );
            });
        } else {
            utils.set_warning_callback(|_, _| {});
        }
        Ok(())
    }

    #[napi]
    pub fn is_overlay_enabled() -> bool {
        crate::client::get_client().utils().is_overlay_enabled()
    }

    #[napi]
    pub fn ui_language() -> String {
        crate::client::get_client().utils().ui_language()
    }

    #[napi]
    pub fn get_app_id() -> u32 {
        let client = crate::client::get_client();
        client.utils().app_id().0
    }

    #[napi]
    pub fn get_server_real_time() -> u32 {
        let client = crate::client::get_client();
        client.utils().get_server_real_time()
    }

    #[napi]
    pub fn is_steam_running_on_steam_deck() -> bool {
        let client = crate::client::get_client();
        client.utils().is_steam_running_on_steam_deck()
    }

    #[napi]
    pub enum GamepadTextInputMode {
        Normal,
        Password,
    }

    #[napi]
    pub enum GamepadTextInputLineMode {
        SingleLine,
        MultipleLines,
    }

    #[napi]
    pub async fn show_gamepad_text_input(
        input_mode: GamepadTextInputMode,
        input_line_mode: GamepadTextInputLineMode,
        description: String,
        max_characters: u32,
        existing_text: Option<String>,
    ) -> Option<String> {
        let _dialog = GAMEPAD_DIALOG.try_lock().ok()?;
        let description = CString::new(description).ok()?;
        let existing_text = existing_text.map(CString::new).transpose().ok()?;
        let client = crate::client::get_client();
        let (tx, rx) = oneshot::channel();
        let mut tx = Some(tx);
        let _dismissed =
            client.register_callback(move |event: steamworks::GamepadTextInputDismissed| {
                if let Some(tx) = tx.take() {
                    let text = event.submitted_text_len.and_then(|length| {
                        let capacity = length.checked_add(1)?;
                        if capacity > 1024 * 1024 {
                            return None;
                        }
                        let mut bytes = vec![0u8; capacity as usize];
                        unsafe {
                            let utils = sys::SteamAPI_SteamUtils_v010();
                            if utils.is_null()
                                || !sys::SteamAPI_ISteamUtils_GetEnteredGamepadTextInput(
                                    utils,
                                    bytes.as_mut_ptr().cast(),
                                    capacity,
                                )
                            {
                                return None;
                            }
                        }
                        CStr::from_bytes_until_nul(&bytes)
                            .ok()?
                            .to_str()
                            .ok()
                            .map(str::to_owned)
                    });
                    let _ = tx.send(text);
                }
            });
        let opened = unsafe {
            let utils = sys::SteamAPI_SteamUtils_v010();
            !utils.is_null()
                && sys::SteamAPI_ISteamUtils_ShowGamepadTextInput(
                    utils,
                    match input_mode {
                        GamepadTextInputMode::Normal => kGamepadTextInputMode::Normal,
                        GamepadTextInputMode::Password => kGamepadTextInputMode::Password,
                    }
                    .into(),
                    match input_line_mode {
                        GamepadTextInputLineMode::SingleLine => {
                            kGamepadTextInputLineMode::SingleLine
                        }
                        GamepadTextInputLineMode::MultipleLines => {
                            kGamepadTextInputLineMode::MultipleLines
                        }
                    }
                    .into(),
                    description.as_ptr(),
                    max_characters,
                    existing_text
                        .as_ref()
                        .map_or(std::ptr::null(), |s| s.as_ptr()),
                )
        };

        if opened {
            rx.await.ok().flatten()
        } else {
            None
        }
    }

    #[napi]
    pub enum FloatingGamepadTextInputMode {
        SingleLine,
        MultipleLines,
        Email,
        Numeric,
    }

    #[napi]
    pub async fn show_floating_gamepad_text_input(
        keyboard_mode: FloatingGamepadTextInputMode,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> bool {
        let Ok(_dialog) = FLOATING_DIALOG.try_lock() else {
            return false;
        };
        let client = crate::client::get_client();
        let (tx, rx) = oneshot::channel();
        let mut tx = Some(tx);
        let _dismissed =
            client.register_callback(move |_: steamworks::FloatingGamepadTextInputDismissed| {
                if let Some(tx) = tx.take() {
                    let _ = tx.send(());
                }
            });
        let opened = unsafe {
            let utils = sys::SteamAPI_SteamUtils_v010();
            !utils.is_null()
                && sys::SteamAPI_ISteamUtils_ShowFloatingGamepadTextInput(
                    utils,
                    match keyboard_mode {
                        FloatingGamepadTextInputMode::SingleLine => {
                            kFloatingGamepadTextInputMode::SingleLine
                        }
                        FloatingGamepadTextInputMode::MultipleLines => {
                            kFloatingGamepadTextInputMode::MultipleLines
                        }
                        FloatingGamepadTextInputMode::Email => kFloatingGamepadTextInputMode::Email,
                        FloatingGamepadTextInputMode::Numeric => {
                            kFloatingGamepadTextInputMode::Numeric
                        }
                    }
                    .into(),
                    x,
                    y,
                    width,
                    height,
                )
        };
        opened && rx.await.is_ok()
    }
}
