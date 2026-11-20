use napi_derive::napi;

#[napi]
pub mod input {
    use napi::bindgen_prelude::{BigInt, Error};
    use std::ffi::CStr;
    use steamworks::sys;

    fn action_handle(value: &BigInt) -> Result<u64, Error> {
        match value.words.as_slice() {
            [word] if !value.sign_bit => Ok(*word),
            [] if !value.sign_bit => Ok(0),
            _ => Err(Error::from_reason(
                "Expected an unsigned 64-bit input handle",
            )),
        }
    }

    fn with_input<T>(call: impl FnOnce(*mut sys::ISteamInput) -> T) -> Result<T, Error> {
        let _client = crate::client::get_client();
        let input = unsafe { sys::SteamAPI_SteamInput_v006() };
        if input.is_null() {
            return Err(Error::from_reason("Steam Input unavailable"));
        }
        Ok(call(input))
    }

    fn action_origin(value: f64) -> Result<sys::EInputActionOrigin, Error> {
        const COUNT: u32 = sys::EInputActionOrigin::k_EInputActionOrigin_Count as u32;
        const _: () = assert!(COUNT == 406);
        if value.fract() != 0.0 || !(0.0..COUNT as f64).contains(&value) {
            return Err(Error::from_reason("Unknown Steam Input action origin"));
        }
        Ok(unsafe { std::mem::transmute::<u32, sys::EInputActionOrigin>(value as u32) })
    }

    unsafe fn copy_text(ptr: *const std::ffi::c_char) -> String {
        if ptr.is_null() {
            String::new()
        } else {
            CStr::from_ptr(ptr).to_string_lossy().into_owned()
        }
    }

    #[napi]
    pub enum GlyphSize {
        Small,
        Medium,
        Large,
    }

    #[napi]
    pub enum HapticLocation {
        Left,
        Right,
        Both,
    }

    #[napi(string_enum)]
    pub enum InputType {
        Unknown,
        SteamController,
        XBox360Controller,
        XBoxOneController,
        GenericGamepad,
        PS4Controller,
        AppleMFiController,
        AndroidController,
        SwitchJoyConPair,
        SwitchJoyConSingle,
        SwitchProController,
        MobileTouch,
        PS3Controller,
        PS5Controller,
        SteamDeckController,
    }

    impl From<steamworks::InputType> for InputType {
        fn from(input_type: steamworks::InputType) -> InputType {
            match input_type {
                steamworks::InputType::Unknown => InputType::Unknown,
                steamworks::InputType::SteamController => InputType::SteamController,
                steamworks::InputType::XBox360Controller => InputType::XBox360Controller,
                steamworks::InputType::XBoxOneController => InputType::XBoxOneController,
                steamworks::InputType::GenericGamepad => InputType::GenericGamepad,
                steamworks::InputType::PS4Controller => InputType::PS4Controller,
                steamworks::InputType::AppleMFiController => InputType::AppleMFiController,
                steamworks::InputType::AndroidController => InputType::AndroidController,
                steamworks::InputType::SwitchJoyConPair => InputType::SwitchJoyConPair,
                steamworks::InputType::SwitchJoyConSingle => InputType::SwitchJoyConSingle,
                steamworks::InputType::SwitchProController => InputType::SwitchProController,
                steamworks::InputType::MobileTouch => InputType::MobileTouch,
                steamworks::InputType::PS3Controller => InputType::PS3Controller,
                steamworks::InputType::PS5Controller => InputType::PS5Controller,
                steamworks::InputType::SteamDeckController => InputType::SteamDeckController,
            }
        }
    }

    #[napi]
    pub struct Controller {
        pub(crate) handle: BigInt,
    }

    impl Controller {
        fn action_origins(
            &self,
            action_set: BigInt,
            action: BigInt,
            get: unsafe extern "C" fn(
                *mut sys::ISteamInput,
                u64,
                u64,
                u64,
                *mut sys::EInputActionOrigin,
            ) -> i32,
        ) -> Result<Vec<u32>, Error> {
            let action_set = action_handle(&action_set)?;
            let action = action_handle(&action)?;
            with_input(|input| {
                let mut origins = [0u32; sys::STEAM_INPUT_MAX_ORIGINS as usize];
                let n = unsafe {
                    get(
                        input,
                        self.handle.get_u64().1,
                        action_set,
                        action,
                        origins.as_mut_ptr().cast(),
                    )
                };
                origins.into_iter().take(n.max(0) as usize).collect()
            })
        }
    }

    #[napi]
    impl Controller {
        #[napi]
        pub fn activate_action_set(&self, action_set_handle: BigInt) {
            let client = crate::client::get_client();
            client
                .input()
                .activate_action_set_handle(self.handle.get_u64().1, action_set_handle.get_u64().1)
        }

        #[napi]
        pub fn is_digital_action_pressed(&self, action_handle: BigInt) -> bool {
            let client = crate::client::get_client();
            client
                .input()
                .get_digital_action_data(self.handle.get_u64().1, action_handle.get_u64().1)
                .bState
        }

        #[napi]
        pub fn get_analog_action_vector(&self, action_handle: BigInt) -> AnalogActionVector {
            let client = crate::client::get_client();
            let data = client
                .input()
                .get_analog_action_data(self.handle.get_u64().1, action_handle.get_u64().1);
            AnalogActionVector {
                x: data.x as f64,
                y: data.y as f64,
            }
        }

        #[napi]
        pub fn get_type(&self) -> InputType {
            let client = crate::client::get_client();
            client
                .input()
                .get_input_type_for_handle(self.handle.get_u64().1)
                .into()
        }

        #[napi]
        pub fn get_handle(&self) -> BigInt {
            self.handle.clone()
        }

        #[napi]
        pub fn get_motion_data(&self) -> MotionData {
            let data = crate::client::get_client()
                .input()
                .get_motion_data(self.handle.get_u64().1);
            MotionData {
                rot_quat_x: data.rotQuatX.into(),
                rot_quat_y: data.rotQuatY.into(),
                rot_quat_z: data.rotQuatZ.into(),
                rot_quat_w: data.rotQuatW.into(),
                pos_accel_x: data.posAccelX.into(),
                pos_accel_y: data.posAccelY.into(),
                pos_accel_z: data.posAccelZ.into(),
                rot_vel_x: data.rotVelX.into(),
                rot_vel_y: data.rotVelY.into(),
                rot_vel_z: data.rotVelZ.into(),
            }
        }

        #[napi]
        pub fn set_lightbar_color(
            &self,
            red: f64,
            green: f64,
            blue: f64,
            restore_default_lightbar_color: Option<bool>,
        ) -> Result<(), Error> {
            if ![red, green, blue]
                .iter()
                .all(|v| v.fract() == 0.0 && (0.0..=255.0).contains(v))
            {
                return Err(Error::from_reason(
                    "RGB values must be integers from 0 to 255",
                ));
            }
            let flag = if restore_default_lightbar_color.unwrap_or(false) {
                sys::ESteamInputLEDFlag::k_ESteamInputLEDFlag_RestoreUserDefault
            } else {
                sys::ESteamInputLEDFlag::k_ESteamInputLEDFlag_SetColor
            };
            with_input(|input| unsafe {
                sys::SteamAPI_ISteamInput_SetLEDColor(
                    input,
                    self.handle.get_u64().1,
                    red as u8,
                    green as u8,
                    blue as u8,
                    flag as u32,
                )
            })
        }

        #[napi]
        pub fn get_digital_action_origins(
            &self,
            action_set: BigInt,
            action: BigInt,
        ) -> Result<Vec<u32>, Error> {
            self.action_origins(
                action_set,
                action,
                sys::SteamAPI_ISteamInput_GetDigitalActionOrigins,
            )
        }

        #[napi]
        pub fn get_analog_action_origins(
            &self,
            action_set: BigInt,
            action: BigInt,
        ) -> Result<Vec<u32>, Error> {
            self.action_origins(
                action_set,
                action,
                sys::SteamAPI_ISteamInput_GetAnalogActionOrigins,
            )
        }

        #[napi]
        pub fn show_binding_panel(&self) -> bool {
            crate::client::get_client()
                .input()
                .show_binding_panel(self.handle.get_u64().1)
        }

        #[napi]
        pub fn get_current_action_set(&self) -> Result<BigInt, Error> {
            with_input(|input| unsafe {
                sys::SteamAPI_ISteamInput_GetCurrentActionSet(input, self.handle.get_u64().1).into()
            })
        }

        #[napi]
        pub fn activate_action_set_layer(&self, layer: BigInt) -> Result<(), Error> {
            let layer = action_handle(&layer)?;
            with_input(|input| unsafe {
                sys::SteamAPI_ISteamInput_ActivateActionSetLayer(
                    input,
                    self.handle.get_u64().1,
                    layer,
                )
            })
        }

        #[napi]
        pub fn deactivate_action_set_layer(&self, layer: BigInt) -> Result<(), Error> {
            let layer = action_handle(&layer)?;
            with_input(|input| unsafe {
                sys::SteamAPI_ISteamInput_DeactivateActionSetLayer(
                    input,
                    self.handle.get_u64().1,
                    layer,
                )
            })
        }

        #[napi]
        pub fn deactivate_all_action_set_layers(&self) -> Result<(), Error> {
            with_input(|input| unsafe {
                sys::SteamAPI_ISteamInput_DeactivateAllActionSetLayers(
                    input,
                    self.handle.get_u64().1,
                )
            })
        }

        #[napi]
        pub fn get_active_action_set_layers(&self) -> Result<Vec<BigInt>, Error> {
            with_input(|input| {
                let mut layers = [0; sys::STEAM_INPUT_MAX_ACTIVE_LAYERS as usize];
                let count = unsafe {
                    sys::SteamAPI_ISteamInput_GetActiveActionSetLayers(
                        input,
                        self.handle.get_u64().1,
                        layers.as_mut_ptr(),
                    )
                };
                layers
                    .into_iter()
                    .take(count.max(0) as usize)
                    .map(BigInt::from)
                    .collect()
            })
        }

        #[napi]
        pub fn trigger_vibration(&self, left_speed: u16, right_speed: u16) -> Result<(), Error> {
            with_input(|input| unsafe {
                sys::SteamAPI_ISteamInput_TriggerVibration(
                    input,
                    self.handle.get_u64().1,
                    left_speed,
                    right_speed,
                )
            })
        }

        #[napi]
        pub fn trigger_simple_haptic_event(
            &self,
            location: HapticLocation,
            intensity: u8,
            gain_db: i8,
            other_intensity: u8,
            other_gain_db: i8,
        ) -> Result<(), Error> {
            let location = match location {
                HapticLocation::Left => {
                    sys::EControllerHapticLocation::k_EControllerHapticLocation_Left
                }
                HapticLocation::Right => {
                    sys::EControllerHapticLocation::k_EControllerHapticLocation_Right
                }
                HapticLocation::Both => {
                    sys::EControllerHapticLocation::k_EControllerHapticLocation_Both
                }
            };
            with_input(|input| unsafe {
                sys::SteamAPI_ISteamInput_TriggerSimpleHapticEvent(
                    input,
                    self.handle.get_u64().1,
                    location,
                    intensity,
                    gain_db as _,
                    other_intensity,
                    other_gain_db as _,
                )
            })
        }
    }

    #[napi(object)]
    pub struct MotionData {
        pub rot_quat_x: f64,
        pub rot_quat_y: f64,
        pub rot_quat_z: f64,
        pub rot_quat_w: f64,
        pub pos_accel_x: f64,
        pub pos_accel_y: f64,
        pub pos_accel_z: f64,
        pub rot_vel_x: f64,
        pub rot_vel_y: f64,
        pub rot_vel_z: f64,
    }

    #[napi(object)]
    pub struct AnalogActionVector {
        pub x: f64,
        pub y: f64,
    }

    #[napi]
    pub fn init() {
        let client = crate::client::get_client();
        client.input().init(false);
    }

    #[napi]
    pub fn get_controllers() -> Vec<Controller> {
        let client = crate::client::get_client();
        let mut handles = [0; sys::STEAM_INPUT_MAX_COUNT as usize];
        let count = client
            .input()
            .get_connected_controllers_slice(&mut handles[..]);
        handles
            .into_iter()
            .take(count)
            .map(|identity| Controller {
                handle: BigInt::from(identity),
            })
            .collect()
    }

    #[napi]
    pub fn get_action_set(action_set_name: String) -> BigInt {
        let client = crate::client::get_client();
        BigInt::from(client.input().get_action_set_handle(&action_set_name))
    }

    #[napi]
    pub fn get_digital_action(action_name: String) -> BigInt {
        let client = crate::client::get_client();
        BigInt::from(client.input().get_digital_action_handle(&action_name))
    }

    #[napi]
    pub fn get_analog_action(action_name: String) -> BigInt {
        let client = crate::client::get_client();
        BigInt::from(client.input().get_analog_action_handle(&action_name))
    }

    #[napi]
    pub fn get_glyph_for_action_origin(origin: f64) -> Result<String, Error> {
        let origin = action_origin(origin)?;
        with_input(|input| unsafe {
            copy_text(sys::SteamAPI_ISteamInput_GetGlyphForActionOrigin_Legacy(
                input, origin,
            ))
        })
    }

    #[napi]
    pub fn get_glyph_png_for_action_origin(
        origin: f64,
        size: GlyphSize,
        flags: Option<u32>,
    ) -> Result<String, Error> {
        let origin = action_origin(origin)?;
        let size = match size {
            GlyphSize::Small => sys::ESteamInputGlyphSize::k_ESteamInputGlyphSize_Small,
            GlyphSize::Medium => sys::ESteamInputGlyphSize::k_ESteamInputGlyphSize_Medium,
            GlyphSize::Large => sys::ESteamInputGlyphSize::k_ESteamInputGlyphSize_Large,
        };
        with_input(|input| unsafe {
            copy_text(sys::SteamAPI_ISteamInput_GetGlyphPNGForActionOrigin(
                input,
                origin,
                size,
                flags.unwrap_or(0),
            ))
        })
    }

    #[napi]
    pub fn get_glyph_svg_for_action_origin(
        origin: f64,
        flags: Option<u32>,
    ) -> Result<String, Error> {
        let origin = action_origin(origin)?;
        with_input(|input| unsafe {
            copy_text(sys::SteamAPI_ISteamInput_GetGlyphSVGForActionOrigin(
                input,
                origin,
                flags.unwrap_or(0),
            ))
        })
    }

    #[napi]
    pub fn get_string_for_action_origin(origin: f64) -> Result<String, Error> {
        let origin = action_origin(origin)?;
        with_input(|input| unsafe {
            copy_text(sys::SteamAPI_ISteamInput_GetStringForActionOrigin(
                input, origin,
            ))
        })
    }

    #[napi]
    pub fn set_input_action_manifest_file_path(path: String) -> Result<bool, Error> {
        if path.contains('\0') {
            return Err(Error::from_reason("Manifest path contains a NUL byte"));
        }
        Ok(crate::client::get_client()
            .input()
            .set_input_action_manifest_file_path(&path))
    }

    #[napi]
    pub fn run_frame() {
        crate::client::get_client().input().run_frame();
    }

    #[napi]
    pub fn enable_device_callbacks() -> Result<(), Error> {
        with_input(|input| unsafe { sys::SteamAPI_ISteamInput_EnableDeviceCallbacks(input) })
    }

    #[napi]
    pub fn shutdown() {
        let client = crate::client::get_client();
        client.input().shutdown()
    }
}
