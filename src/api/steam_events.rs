use steamworks::{sys, Callback};

macro_rules! scalar_event {
    ($name:ident, $field:ident: $ty:ty, $raw:ident, $id:ident, $source:ident) => {
        #[derive(serde::Serialize)]
        pub(super) struct $name {
            $field: $ty,
        }

        unsafe impl Callback for $name {
            const ID: i32 = sys::$id as i32;
            const SIZE: i32 = std::mem::size_of::<sys::$raw>() as i32;
            unsafe fn from_raw(raw: *mut std::ffi::c_void) -> Self {
                Self {
                    $field: raw.cast::<sys::$raw>().read_unaligned().$source,
                }
            }
        }
    };
}

scalar_event!(DeviceConnected, device_handle: u64, SteamInputDeviceConnected_t,
    SteamInputDeviceConnected_t_k_iCallback, m_ulConnectedDeviceHandle);
scalar_event!(DeviceDisconnected, device_handle: u64, SteamInputDeviceDisconnected_t,
    SteamInputDeviceDisconnected_t_k_iCallback, m_ulDisconnectedDeviceHandle);
scalar_event!(InventoryFullUpdate, handle: i32, SteamInventoryFullUpdate_t,
    SteamInventoryFullUpdate_t_k_iCallback, m_handle);

#[derive(serde::Serialize)]
pub(super) struct InventoryDefinitionUpdate;

unsafe impl Callback for InventoryDefinitionUpdate {
    const ID: i32 = sys::SteamInventoryDefinitionUpdate_t_k_iCallback as i32;
    const SIZE: i32 = std::mem::size_of::<sys::SteamInventoryDefinitionUpdate_t>() as i32;
    unsafe fn from_raw(_: *mut std::ffi::c_void) -> Self {
        Self
    }
}

#[napi_derive::napi(object, object_from_js = false)]
pub struct TicketValidation {
    #[napi(js_name = "steam_id")]
    pub peer: u64,
    #[napi(js_name = "owner_steam_id")]
    pub owner: u64,
    pub response: u32,
}

unsafe impl steamworks::Callback for TicketValidation {
    const ID: i32 = sys::ValidateAuthTicketResponse_t_k_iCallback as i32;
    const SIZE: i32 = std::mem::size_of::<sys::ValidateAuthTicketResponse_t>() as i32;
    unsafe fn from_raw(raw: *mut std::ffi::c_void) -> Self {
        let event = raw.cast::<sys::ValidateAuthTicketResponse_t>();
        Self {
            peer: std::ptr::addr_of!((*event).m_SteamID.m_steamid.m_unAll64Bits).read_unaligned(),
            owner: std::ptr::addr_of!((*event).m_OwnerSteamID.m_steamid.m_unAll64Bits)
                .read_unaligned(),
            response: std::ptr::addr_of!((*event).m_eAuthSessionResponse)
                .cast::<u32>()
                .read_unaligned(),
        }
    }
}
