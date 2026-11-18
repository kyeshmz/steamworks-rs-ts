use napi_derive::napi;

#[napi]
pub mod inventory {
    use napi::bindgen_prelude::{Error, Result};
    use std::ffi::{CStr, CString};
    use std::ptr;
    use steamworks::sys;

    fn with_inventory<T>(read: impl FnOnce(*mut sys::ISteamInventory) -> T) -> Result<T> {
        let _client = crate::client::get_client();
        let interface = unsafe { sys::SteamAPI_SteamInventory_v003() };
        if interface.is_null() {
            return Err(Error::from_reason("Steam inventory is unavailable"));
        }
        Ok(read(interface))
    }

    /// Requests a refresh. A successful request does not mean definitions are ready.
    #[napi]
    pub fn load_item_definitions() -> Result<bool> {
        with_inventory(|inv| unsafe { sys::SteamAPI_ISteamInventory_LoadItemDefinitions(inv) })
    }

    /// Returns the cached definition IDs, or null if Steam could not read them.
    #[napi]
    pub fn get_item_definition_ids() -> Result<Option<Vec<i32>>> {
        with_inventory(|inv| {
            read_array(|data, count| unsafe {
                sys::SteamAPI_ISteamInventory_GetItemDefinitionIDs(inv, data, count)
            })
        })
    }

    /// Omit name to read the comma-separated property names. Missing properties return null.
    #[napi]
    pub fn get_item_definition_property(
        definition: i32,
        name: Option<String>,
    ) -> Result<Option<String>> {
        let name = name
            .map(CString::new)
            .transpose()
            .map_err(|_| Error::from_reason("Inventory property name contains a NUL byte"))?;
        with_inventory(|inv| {
            let bytes = read_array(|data: *mut u8, size| unsafe {
                sys::SteamAPI_ISteamInventory_GetItemDefinitionProperty(
                    inv,
                    definition,
                    name.as_ref().map_or(ptr::null(), |s| s.as_ptr()),
                    data.cast(),
                    size,
                )
            })?;
            Some(
                CStr::from_bytes_until_nul(&bytes)
                    .ok()?
                    .to_string_lossy()
                    .into_owned(),
            )
        })
    }

    /// Records play time for playtime-based item drops. Does not grant an item.
    #[napi]
    pub fn send_item_drop_heartbeat() -> Result<()> {
        with_inventory(|inv| unsafe { sys::SteamAPI_ISteamInventory_SendItemDropHeartbeat(inv) })
    }

    fn read_array<T: Copy + Default>(read: impl Fn(*mut T, &mut u32) -> bool) -> Option<Vec<T>> {
        let mut count = 0;
        if !read(ptr::null_mut(), &mut count) {
            return None;
        }
        let mut values = Vec::new();
        if count > 0 {
            values.try_reserve_exact(count as usize).ok()?;
            values.resize(count as usize, T::default());
            if !read(values.as_mut_ptr(), &mut count) || count as usize > values.len() {
                return None;
            }
            values.truncate(count as usize);
        }
        Some(values)
    }
}
