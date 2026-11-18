use napi_derive::napi;

#[napi]
pub mod inventory {
    use napi::bindgen_prelude::{BigInt, Buffer, Error, Result};
    use std::collections::HashMap;
    use std::ffi::{CStr, CString};
    use std::ptr;
    use steamworks::sys;
    use tokio::time::{sleep, Duration, Instant};

    #[napi(object)]
    pub struct InventoryItem {
        pub item_id: BigInt,
        pub definition: i32,
        pub quantity: u32,
        pub flags: u32,
    }

    #[napi(object)]
    pub struct InventoryResult {
        pub timestamp: u32,
        pub items: Vec<InventoryItem>,
    }

    #[napi(object)]
    pub struct ItemProperties {
        pub item_id: BigInt,
        pub definition: i32,
        pub quantity: u32,
        pub properties: HashMap<String, String>,
    }

    #[napi(object)]
    pub struct ExchangeGenerate {
        pub definition: f64,
        pub quantity: f64,
    }

    #[napi(object)]
    pub struct ExchangeDestroy {
        pub item_id: BigInt,
        pub quantity: f64,
    }

    struct ResultHandle(sys::SteamInventoryResult_t);

    #[napi(object)]
    pub struct SerializedResult {
        pub timestamp: u32,
        pub items: Vec<InventoryItem>,
        pub buffer: Buffer,
    }

    impl Drop for ResultHandle {
        fn drop(&mut self) {
            // init never replaces the process client, even while this request awaits Steam.
            unsafe {
                sys::SteamAPI_ISteamInventory_DestroyResult(
                    sys::SteamAPI_SteamInventory_v003(),
                    self.0,
                )
            }
        }
    }

    fn instance_id(value: &BigInt) -> Result<u64> {
        match (value.sign_bit, value.words.as_slice()) {
            (false, []) => Ok(0),
            (false, [word]) => Ok(*word),
            _ => Err(Error::from_reason(
                "Inventory instance ID must be an unsigned 64-bit bigint",
            )),
        }
    }

    // k_SteamItemInstanceIDInvalid is all bits set, so no real item ID uses every bit.
    fn mutation_id(value: &BigInt) -> Result<u64> {
        match instance_id(value)? {
            u64::MAX => Err(Error::from_reason(
                "Inventory instance ID cannot be the Steam invalid sentinel",
            )),
            id => Ok(id),
        }
    }

    fn submit(
        request: impl FnOnce(
            *mut sys::ISteamInventory,
            &mut sys::SteamInventoryResult_t,
        ) -> Result<bool>,
    ) -> Result<ResultHandle> {
        with_inventory(|inv| {
            let mut handle = sys::k_SteamInventoryResultInvalid;
            let submitted = request(inv, &mut handle)?;
            let result =
                (handle != sys::k_SteamInventoryResultInvalid).then(|| ResultHandle(handle));
            result
                .filter(|_| submitted)
                .ok_or_else(|| Error::from_reason("Steam inventory request was not submitted"))
        })?
    }

    async fn await_ready(handle: ResultHandle) -> Result<ResultHandle> {
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            let status = unsafe {
                sys::SteamAPI_ISteamInventory_GetResultStatus(
                    sys::SteamAPI_SteamInventory_v003(),
                    handle.0,
                )
            };
            match status {
                sys::EResult::k_EResultOK => return Ok(handle),
                sys::EResult::k_EResultPending if Instant::now() < deadline => {
                    sleep(Duration::from_millis(50)).await
                }
                sys::EResult::k_EResultPending => {
                    return Err(Error::from_reason("Steam inventory request timed out"))
                }
                status => {
                    return Err(Error::from_reason(
                        format!("{status:?}").trim_start_matches("k_EResult"),
                    ))
                }
            }
        }
    }

    async fn ready(ids: Option<Vec<BigInt>>) -> Result<ResultHandle> {
        let ids = ids
            .map(|ids| {
                ids.into_iter()
                    .map(|id| instance_id(&id))
                    .collect::<Result<Vec<_>>>()
            })
            .transpose()?;
        let handle = submit(|inv, handle| {
            Ok(match ids {
                Some(ids) => unsafe {
                    sys::SteamAPI_ISteamInventory_GetItemsByID(
                        inv,
                        handle,
                        ids.as_ptr(),
                        ids.len()
                            .try_into()
                            .map_err(|_| Error::from_reason("Too many inventory instance IDs"))?,
                    )
                },
                None => unsafe { sys::SteamAPI_ISteamInventory_GetAllItems(inv, handle) },
            })
        })?;
        await_ready(handle).await
    }

    impl ResultHandle {
        fn items(&self) -> Result<Vec<InventoryItem>> {
            let empty = sys::SteamItemDetails_t {
                m_itemId: 0,
                m_iDefinition: 0,
                m_unQuantity: 0,
                m_unFlags: 0,
            };
            let rows = read_array(empty, |data, size| unsafe {
                sys::SteamAPI_ISteamInventory_GetResultItems(
                    sys::SteamAPI_SteamInventory_v003(),
                    self.0,
                    data,
                    size,
                )
            })
            .ok_or_else(|| Error::from_reason("Could not read inventory result items"))?;
            Ok(rows
                .into_iter()
                .map(|row| InventoryItem {
                    item_id: row.m_itemId.into(),
                    definition: row.m_iDefinition,
                    quantity: row.m_unQuantity.into(),
                    flags: row.m_unFlags.into(),
                })
                .collect())
        }

        fn snapshot(&self) -> Result<InventoryResult> {
            Ok(InventoryResult {
                timestamp: unsafe {
                    sys::SteamAPI_ISteamInventory_GetResultTimestamp(
                        sys::SteamAPI_SteamInventory_v003(),
                        self.0,
                    )
                },
                items: self.items()?,
            })
        }

        fn property(&self, index: u32, name: Option<&CStr>) -> Result<String> {
            let bytes = read_array(0u8, |buf, size| unsafe {
                sys::SteamAPI_ISteamInventory_GetResultItemProperty(
                    sys::SteamAPI_SteamInventory_v003(),
                    self.0,
                    index,
                    name.map_or(ptr::null(), CStr::as_ptr),
                    buf.cast(),
                    size,
                )
            });
            bytes
                .as_deref()
                .and_then(|b| CStr::from_bytes_until_nul(b).ok())
                .map(|s| s.to_string_lossy().into_owned())
                .ok_or_else(|| Error::from_reason("Could not read inventory item property"))
        }

        fn properties(&self) -> Result<Vec<ItemProperties>> {
            self.items()?
                .into_iter()
                .enumerate()
                .map(|(i, item)| {
                    let index = i as u32;
                    let mut properties = HashMap::new();
                    for key in self
                        .property(index, None)?
                        .split(',')
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                    {
                        let name = CString::new(key)
                            .map_err(|_| Error::from_reason("Invalid property name"))?;
                        properties.insert(key.to_string(), self.property(index, Some(&name))?);
                    }
                    Ok(ItemProperties {
                        item_id: item.item_id,
                        definition: item.definition,
                        quantity: item.quantity,
                        properties,
                    })
                })
                .collect()
        }

        fn receipt(&self) -> Result<SerializedResult> {
            let bytes = read_array(0u8, |data, size| unsafe {
                sys::SteamAPI_ISteamInventory_SerializeResult(
                    sys::SteamAPI_SteamInventory_v003(),
                    self.0,
                    data.cast(),
                    size,
                )
            })
            .ok_or_else(|| Error::from_reason("Could not serialize the inventory result"))?;
            let snapshot = self.snapshot()?;
            Ok(SerializedResult {
                timestamp: snapshot.timestamp,
                items: snapshot.items,
                buffer: Buffer::from(bytes),
            })
        }
    }

    #[napi]
    pub async fn get_all_items() -> Result<InventoryResult> {
        ready(None).await?.snapshot()
    }

    #[napi]
    pub async fn get_items_by_id(item_ids: Vec<BigInt>) -> Result<InventoryResult> {
        ready(Some(item_ids)).await?.snapshot()
    }

    #[napi(js_name = "getItemsByID")]
    pub async fn get_items_by_id_sdk(item_ids: Vec<BigInt>) -> Result<InventoryResult> {
        get_items_by_id(item_ids).await
    }

    #[napi]
    pub async fn get_item_properties(item_ids: Vec<BigInt>) -> Result<Vec<ItemProperties>> {
        ready(Some(item_ids)).await?.properties()
    }

    #[napi]
    pub async fn get_all_item_properties() -> Result<Vec<ItemProperties>> {
        ready(None).await?.properties()
    }

    struct ExchangePlan {
        definitions: Vec<i32>,
        generate_quantities: Vec<u32>,
        ids: Vec<u64>,
        destroy_quantities: Vec<u32>,
    }

    fn checked_quantity(value: f64) -> Result<u32> {
        if value.fract() == 0.0 && (1.0..=f64::from(u32::MAX)).contains(&value) {
            Ok(value as u32)
        } else {
            Err(Error::from_reason(
                "Quantity must be an unsigned 32-bit integer greater than zero",
            ))
        }
    }

    impl ExchangePlan {
        fn parse(generate: Vec<ExchangeGenerate>, destroy: Vec<ExchangeDestroy>) -> Result<Self> {
            if generate.is_empty() && destroy.is_empty() {
                return Err(Error::from_reason(
                    "Exchange needs at least one generated definition or destroyed item",
                ));
            }
            for entry in &generate {
                if entry.definition.fract() != 0.0
                    || !(1.0..=f64::from(i32::MAX)).contains(&entry.definition)
                {
                    return Err(Error::from_reason(
                        "Expected a positive 32-bit item definition",
                    ));
                }
            }
            let mut ids = Vec::with_capacity(destroy.len());
            for entry in &destroy {
                ids.push(mutation_id(&entry.item_id)?);
            }
            Ok(ExchangePlan {
                definitions: generate
                    .iter()
                    .map(|entry| entry.definition as i32)
                    .collect(),
                generate_quantities: generate
                    .iter()
                    .map(|entry| checked_quantity(entry.quantity))
                    .collect::<Result<_>>()?,
                ids,
                destroy_quantities: destroy
                    .iter()
                    .map(|entry| checked_quantity(entry.quantity))
                    .collect::<Result<_>>()?,
            })
        }
    }

    fn submit_exchange(plan: &ExchangePlan) -> Result<ResultHandle> {
        submit(|inv, handle| {
            let generate_len = u32::try_from(plan.definitions.len())
                .map_err(|_| Error::from_reason("Too many items in one exchange"))?;
            let destroy_len = u32::try_from(plan.ids.len())
                .map_err(|_| Error::from_reason("Too many items in one exchange"))?;
            Ok(unsafe {
                sys::SteamAPI_ISteamInventory_ExchangeItems(
                    inv,
                    handle,
                    plan.definitions.as_ptr(),
                    plan.generate_quantities.as_ptr(),
                    generate_len,
                    plan.ids.as_ptr(),
                    plan.destroy_quantities.as_ptr(),
                    destroy_len,
                )
            })
        })
    }

    /// Exchanges item quantities in one atomic Steam transaction and returns the resulting snapshot.
    #[napi]
    pub async fn exchange_items(
        generate: Vec<ExchangeGenerate>,
        destroy: Vec<ExchangeDestroy>,
    ) -> Result<InventoryResult> {
        let plan = ExchangePlan::parse(generate, destroy)?;
        let handle = submit_exchange(&plan)?;
        await_ready(handle).await?.snapshot()
    }

    /// Consumes item quantity permanently and returns the Steam-signed receipt bytes.
    #[napi]
    pub async fn consume_item_serialized(
        item_id: BigInt,
        quantity: f64,
    ) -> Result<SerializedResult> {
        let item = mutation_id(&item_id)?;
        let quantity = checked_quantity(quantity)?;
        let handle = submit(|inv, out| {
            Ok(unsafe { sys::SteamAPI_ISteamInventory_ConsumeItem(inv, out, item, quantity) })
        })?;
        await_ready(handle).await?.receipt()
    }

    /// Exchanges items in one atomic Steam transaction and returns the Steam-signed receipt bytes.
    #[napi]
    pub async fn exchange_items_serialized(
        generate: Vec<ExchangeGenerate>,
        destroy: Vec<ExchangeDestroy>,
    ) -> Result<SerializedResult> {
        let plan = ExchangePlan::parse(generate, destroy)?;
        let handle = submit_exchange(&plan)?;
        await_ready(handle).await?.receipt()
    }

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
            read_array(0, |data, count| unsafe {
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
            let bytes = read_array(0u8, |data, size| unsafe {
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

    fn read_array<T: Copy>(empty: T, read: impl Fn(*mut T, &mut u32) -> bool) -> Option<Vec<T>> {
        let mut count = 0;
        if !read(ptr::null_mut(), &mut count) {
            return None;
        }
        let mut values = Vec::new();
        if count > 0 {
            if count as usize > (64 * 1024 * 1024) / std::mem::size_of::<T>() {
                return None;
            }
            values.try_reserve_exact(count as usize).ok()?;
            values.resize(count as usize, empty);
            if !read(values.as_mut_ptr(), &mut count) || count as usize > values.len() {
                return None;
            }
            values.truncate(count as usize);
        }
        Some(values)
    }
}
