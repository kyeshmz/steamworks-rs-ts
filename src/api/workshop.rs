use napi_derive::napi;

#[napi]
pub mod workshop {
    use napi::bindgen_prelude::{BigInt, Error, Function};
    use napi::threadsafe_function::ThreadsafeFunctionCallMode;
    use std::path::Path;
    use steamworks::{PublishedFileId, UpdateHandle};
    use tokio::sync::oneshot;

    const MAX_SUBSCRIBED_ITEMS: u32 = 100_000;
    const MAX_PLAYTIME_ITEMS: usize = 100;

    pub(crate) fn published_file_id(id: &BigInt) -> Result<PublishedFileId, Error> {
        match id.words.as_slice() {
            &[value] if !id.sign_bit && value != 0 => Ok(PublishedFileId(value)),
            _ => Err(Error::from_reason(
                "Workshop item ID must be a positive unsigned 64-bit integer",
            )),
        }
    }

    #[napi(object)]
    pub struct UgcResult {
        pub item_id: BigInt,
        pub needs_to_accept_agreement: bool,
    }

    #[derive(Debug)]
    #[napi]
    pub enum UgcItemVisibility {
        Public,
        FriendsOnly,
        Private,
        Unlisted,
    }

    impl From<steamworks::PublishedFileVisibility> for UgcItemVisibility {
        fn from(visibility: steamworks::PublishedFileVisibility) -> Self {
            match visibility {
                steamworks::PublishedFileVisibility::Public => UgcItemVisibility::Public,
                steamworks::PublishedFileVisibility::FriendsOnly => UgcItemVisibility::FriendsOnly,
                steamworks::PublishedFileVisibility::Private => UgcItemVisibility::Private,
                steamworks::PublishedFileVisibility::Unlisted => UgcItemVisibility::Unlisted,
            }
        }
    }

    impl From<UgcItemVisibility> for steamworks::PublishedFileVisibility {
        fn from(val: UgcItemVisibility) -> Self {
            match val {
                UgcItemVisibility::Public => steamworks::PublishedFileVisibility::Public,
                UgcItemVisibility::FriendsOnly => steamworks::PublishedFileVisibility::FriendsOnly,
                UgcItemVisibility::Private => steamworks::PublishedFileVisibility::Private,
                UgcItemVisibility::Unlisted => steamworks::PublishedFileVisibility::Unlisted,
            }
        }
    }

    #[derive(Debug)]
    #[napi(object)]
    pub struct KeyValueTag {
        pub key: String,
        pub value: String,
    }

    fn validate_key(key: &str) -> Result<(), Error> {
        if key.is_empty()
            || key.len() > 255
            || !key.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
            return Err(Error::from_reason(
                "Workshop tag keys must be 1 to 255 ASCII letters, digits, or underscores",
            ));
        }
        Ok(())
    }

    impl KeyValueTag {
        pub(crate) fn validate(&self) -> Result<(), Error> {
            validate_key(&self.key)?;
            if self.value.len() > 255 || self.value.contains('\0') {
                return Err(Error::from_reason(
                    "Workshop tag values must be at most 255 UTF-8 bytes without NUL",
                ));
            }
            Ok(())
        }
    }

    #[napi]
    pub enum ContentDescriptor {
        NudityOrSexualContent,
        FrequentViolenceOrGore,
        AdultOnlySexualContent,
        GratuitousSexualContent,
        AnyMatureContent,
    }

    impl From<ContentDescriptor> for steamworks::UGCContentDescriptorID {
        fn from(value: ContentDescriptor) -> Self {
            match value {
                ContentDescriptor::NudityOrSexualContent => Self::NudityOrSexualContent,
                ContentDescriptor::FrequentViolenceOrGore => Self::FrequentViolenceOrGore,
                ContentDescriptor::AdultOnlySexualContent => Self::AdultOnlySexualContent,
                ContentDescriptor::GratuitousSexualContent => Self::GratuitousSexualContent,
                ContentDescriptor::AnyMatureContent => Self::AnyMatureContent,
            }
        }
    }

    #[napi(object)]
    pub struct UgcUpdate {
        pub title: Option<String>,
        pub description: Option<String>,
        pub language: Option<String>,
        pub change_note: Option<String>,
        pub preview_path: Option<String>,
        pub content_path: Option<String>,
        pub tags: Option<Vec<String>>,
        pub visibility: Option<UgcItemVisibility>,
        pub metadata: Option<String>,
        pub key_value_tags: Option<Vec<KeyValueTag>>,
        pub remove_key_value_tags: Option<Vec<String>>,
        pub remove_all_key_value_tags: Option<bool>,
        pub allow_admin_tags: Option<bool>,
        pub content_descriptors: Option<Vec<ContentDescriptor>>,
        pub remove_content_descriptors: Option<Vec<ContentDescriptor>>,
    }

    impl UgcUpdate {
        fn validate(&mut self) -> Result<(), Error> {
            if self
                .title
                .iter()
                .chain(&self.description)
                .chain(&self.language)
                .chain(&self.change_note)
                .chain(&self.preview_path)
                .chain(&self.content_path)
                .chain(self.tags.iter().flatten())
                .any(|s| s.contains('\0'))
            {
                return Err(Error::from_reason(
                    "Workshop update strings must not contain NUL",
                ));
            }
            if let Some(metadata) = &self.metadata {
                if metadata.contains('\0')
                    || metadata.len() >= steamworks::sys::k_cchDeveloperMetadataMax as usize
                {
                    return Err(Error::from_reason(
                        "Workshop metadata must fit in 4999 UTF-8 bytes without NUL",
                    ));
                }
            }
            if let Some(tags) = &self.key_value_tags {
                if tags.len() > 100 {
                    return Err(Error::from_reason(
                        "At most 100 key/value tags may be added per update",
                    ));
                }
                for tag in tags {
                    tag.validate()?;
                }
            }
            for key in self.remove_key_value_tags.iter().flatten() {
                validate_key(key)?;
            }
            for value in [&mut self.preview_path, &mut self.content_path]
                .into_iter()
                .flatten()
            {
                *value = Path::new(value)
                    .canonicalize()
                    .map_err(|e| Error::from_reason(format!("Invalid workshop path: {e}")))?
                    .to_string_lossy()
                    .into_owned();
            }
            Ok(())
        }

        fn prepare(
            self,
            mut update: UpdateHandle,
        ) -> Result<(UpdateHandle, Option<String>), Error> {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if let Some(title) = self.title {
                    update = update.title(title.as_str());
                }

                if let Some(description) = self.description {
                    update = update.description(description.as_str());
                }

                if let Some(language) = self.language {
                    update = update.language(&language);
                }

                if let Some(preview_path) = self.preview_path {
                    update = update.preview_path(Path::new(&preview_path));
                }

                if let Some(tags) = self.tags {
                    update = update.tags(tags, self.allow_admin_tags.unwrap_or(false));
                }

                if let Some(content_path) = self.content_path {
                    update = update.content_path(Path::new(&content_path));
                }

                if let Some(visibility) = self.visibility {
                    update = update.visibility(visibility.into());
                }

                if let Some(metadata) = self.metadata {
                    update = update.metadata(&metadata);
                }
                if self.remove_all_key_value_tags.unwrap_or(false) {
                    update = update.remove_all_key_value_tags();
                }
                for key in self.remove_key_value_tags.into_iter().flatten() {
                    update = update.remove_key_value_tag(&key);
                }
                for tag in self.key_value_tags.into_iter().flatten() {
                    update = update.add_key_value_tag(&tag.key, &tag.value);
                }
                for descriptor in self.remove_content_descriptors.into_iter().flatten() {
                    update = update.remove_content_descriptor(descriptor.into());
                }
                for descriptor in self.content_descriptors.into_iter().flatten() {
                    update = update.add_content_descriptor(descriptor.into());
                }

                (update, self.change_note)
            }))
            .map_err(|_| Error::from_reason("Steam rejected workshop update fields"))
        }
    }

    #[napi(object)]
    pub struct InstallInfo {
        pub folder: String,
        pub size_on_disk: BigInt,
        pub timestamp: u32,
    }

    #[napi(object)]
    pub struct DownloadInfo {
        pub current: BigInt,
        pub total: BigInt,
    }

    #[derive(Debug)]
    #[napi]
    pub enum UpdateStatus {
        Invalid,
        PreparingConfig,
        PreparingContent,
        UploadingContent,
        UploadingPreviewFile,
        CommittingChanges,
    }

    impl From<steamworks::UpdateStatus> for UpdateStatus {
        fn from(visibility: steamworks::UpdateStatus) -> Self {
            match visibility {
                steamworks::UpdateStatus::Invalid => UpdateStatus::Invalid,
                steamworks::UpdateStatus::PreparingConfig => UpdateStatus::PreparingConfig,
                steamworks::UpdateStatus::PreparingContent => UpdateStatus::PreparingContent,
                steamworks::UpdateStatus::UploadingContent => UpdateStatus::UploadingContent,
                steamworks::UpdateStatus::UploadingPreviewFile => {
                    UpdateStatus::UploadingPreviewFile
                }
                steamworks::UpdateStatus::CommittingChanges => UpdateStatus::CommittingChanges,
            }
        }
    }

    #[napi(object)]
    pub struct UpdateProgress {
        pub status: UpdateStatus,
        pub progress: BigInt,
        pub total: BigInt,
    }

    #[napi]
    pub enum FileType {
        Community,
        Microtransaction,
        Collection,
        Art,
        Video,
        Screenshot,
        Game,
        Software,
        Concept,
        WebGuide,
        IntegratedGuide,
        Merch,
        ControllerBinding,
        SteamworksAccessInvite,
        SteamVideo,
        GameManagedItem,
    }

    impl From<FileType> for steamworks::FileType {
        fn from(file_type: FileType) -> steamworks::FileType {
            match file_type {
                FileType::Community => steamworks::FileType::Community,
                FileType::Microtransaction => steamworks::FileType::Microtransaction,
                FileType::Collection => steamworks::FileType::Collection,
                FileType::Art => steamworks::FileType::Art,
                FileType::Video => steamworks::FileType::Video,
                FileType::Screenshot => steamworks::FileType::Screenshot,
                FileType::Game => steamworks::FileType::Game,
                FileType::Software => steamworks::FileType::Software,
                FileType::Concept => steamworks::FileType::Concept,
                FileType::WebGuide => steamworks::FileType::WebGuide,
                FileType::IntegratedGuide => steamworks::FileType::IntegratedGuide,
                FileType::Merch => steamworks::FileType::Merch,
                FileType::ControllerBinding => steamworks::FileType::ControllerBinding,
                FileType::SteamworksAccessInvite => steamworks::FileType::SteamworksAccessInvite,
                FileType::SteamVideo => steamworks::FileType::SteamVideo,
                FileType::GameManagedItem => steamworks::FileType::GameManagedItem,
            }
        }
    }

    #[napi]
    pub async fn create_item(
        app_id: Option<u32>,
        file_type: Option<FileType>,
    ) -> Result<UgcResult, Error> {
        let client = crate::client::get_client();
        let app_id = app_id
            .map(steamworks::AppId)
            .unwrap_or_else(|| client.utils().app_id());
        let file_type = file_type.unwrap_or(FileType::Community);

        let (tx, rx) = oneshot::channel();

        crate::api::runtime::without_dispatch(|| {
            client
                .ugc()
                .create_item(app_id, file_type.into(), move |result| {
                    let _ = tx.send(result);
                });
        });

        let result = tokio::time::timeout(std::time::Duration::from_secs(30), rx)
            .await
            .map_err(|_| Error::from_reason("Timed out waiting for create item callback"))?
            .map_err(|_| Error::from_reason("Create item callback dropped"))?;
        match result {
            Ok((item_id, needs_to_accept_agreement)) => Ok(UgcResult {
                item_id: BigInt::from(item_id.0),
                needs_to_accept_agreement,
            }),
            Err(e) => Err(Error::from_reason(e.to_string())),
        }
    }

    #[napi]
    pub async fn update_item(
        item_id: BigInt,
        mut update_details: UgcUpdate,
        app_id: Option<u32>,
    ) -> Result<UgcResult, Error> {
        update_details.validate()?;
        let item_id = published_file_id(&item_id)?;
        let client = crate::client::get_client();

        let app_id = app_id
            .map(steamworks::AppId)
            .unwrap_or_else(|| client.utils().app_id());

        let update_handle = client.ugc().start_item_update(app_id, item_id);
        let (update_handle, change_note) = update_details.prepare(update_handle)?;
        let (tx, rx) = oneshot::channel();
        crate::api::runtime::without_dispatch(|| {
            update_handle.submit(change_note.as_deref(), move |result| {
                let _ = tx.send(result);
            });
        });

        let result = tokio::time::timeout(std::time::Duration::from_secs(30), rx)
            .await
            .map_err(|_| Error::from_reason("Timed out waiting for update callback"))?
            .map_err(|_| Error::from_reason("Update callback dropped"))?;
        match result {
            Ok((item_id, needs_to_accept_agreement)) => Ok(UgcResult {
                item_id: BigInt::from(item_id.0),
                needs_to_accept_agreement,
            }),
            Err(e) => Err(Error::from_reason(e.to_string())),
        }
    }

    #[napi]
    pub fn update_item_with_callback(
        item_id: BigInt,
        mut update_details: UgcUpdate,
        app_id: Option<u32>,

        #[napi(ts_arg_type = "(data: UgcResult) => void")] success_callback: Function<
            '_,
            UgcResult,
            (),
        >,

        #[napi(ts_arg_type = "(err: any) => void")] error_callback: Function<'_, Error, ()>,

        #[napi(ts_arg_type = "(data: UpdateProgress) => void")] progress_callback: Option<
            Function<'_, UpdateProgress, ()>,
        >,

        progress_callback_interval_ms: Option<u32>,
    ) {
        let success_callback = success_callback
            .build_threadsafe_function()
            .build()
            .unwrap();
        let error_callback = error_callback.build_threadsafe_function().build().unwrap();

        let item_id = match update_details
            .validate()
            .and_then(|()| published_file_id(&item_id))
        {
            Ok(id) => id,
            Err(error) => {
                error_callback.call(error, ThreadsafeFunctionCallMode::NonBlocking);
                return;
            }
        };
        let client = crate::client::get_client();

        let app_id = app_id
            .map(steamworks::AppId)
            .unwrap_or_else(|| client.utils().app_id());

        {
            let update_handle = client.ugc().start_item_update(app_id, item_id);
            let (update_handle, change_note) = match update_details.prepare(update_handle) {
                Ok(prepared) => prepared,
                Err(error) => {
                    error_callback.call(error, ThreadsafeFunctionCallMode::NonBlocking);
                    return;
                }
            };
            let update_watch_handle = crate::api::runtime::without_dispatch(|| {
                update_handle.submit(change_note.as_deref(), move |result| {
                    match result {
                        Ok((item_id, needs_to_accept_agreement)) => success_callback.call(
                            UgcResult {
                                item_id: BigInt::from(item_id.0),
                                needs_to_accept_agreement,
                            },
                            ThreadsafeFunctionCallMode::Blocking,
                        ),
                        Err(e) => error_callback.call(
                            Error::from_reason(e.to_string()),
                            ThreadsafeFunctionCallMode::Blocking,
                        ),
                    };
                })
            });

            if let Some(progress_callback) = progress_callback {
                let progress_callback = progress_callback
                    .build_threadsafe_function()
                    .build()
                    .unwrap();

                std::thread::spawn(move || loop {
                    let (status, progress, total) = update_watch_handle.progress();
                    let value = UpdateProgress {
                        status: status.into(),
                        progress: BigInt::from(progress),
                        total: BigInt::from(total),
                    };
                    progress_callback.call(value, ThreadsafeFunctionCallMode::Blocking);
                    match status {
                        steamworks::UpdateStatus::Invalid => break,
                        steamworks::UpdateStatus::CommittingChanges => break,
                        _ => (),
                    }
                    std::thread::sleep(std::time::Duration::from_millis(
                        progress_callback_interval_ms.unwrap_or(1000) as u64,
                    ));
                });
            }
        };
    }

    #[napi]
    pub async fn subscribe(item_id: BigInt) -> Result<(), Error> {
        let item_id = published_file_id(&item_id)?;
        let client = crate::client::get_client();
        let (tx, rx) = oneshot::channel();

        crate::api::runtime::without_dispatch(|| {
            client.ugc().subscribe_item(item_id, move |result| {
                let _ = tx.send(result);
            });
        });

        let result = rx
            .await
            .map_err(|_| Error::from_reason("Subscribe callback dropped"))?;
        match result {
            Ok(()) => Ok(()),
            Err(e) => Err(Error::from_reason(e.to_string())),
        }
    }

    #[napi]
    pub async fn unsubscribe(item_id: BigInt) -> Result<(), Error> {
        let item_id = published_file_id(&item_id)?;
        let client = crate::client::get_client();
        let (tx, rx) = oneshot::channel();

        crate::api::runtime::without_dispatch(|| {
            client.ugc().unsubscribe_item(item_id, move |result| {
                let _ = tx.send(result);
            });
        });

        let result = rx
            .await
            .map_err(|_| Error::from_reason("Unsubscribe callback dropped"))?;
        match result {
            Ok(()) => Ok(()),
            Err(e) => Err(Error::from_reason(e.to_string())),
        }
    }

    #[napi]
    pub fn state(item_id: BigInt) -> Result<u32, Error> {
        let item_id = published_file_id(&item_id)?;
        let client = crate::client::get_client();
        let result = client.ugc().item_state(item_id);

        Ok(result.bits())
    }

    #[napi]
    pub fn install_info(item_id: BigInt) -> Result<Option<InstallInfo>, Error> {
        let item_id = published_file_id(&item_id)?;
        let client = crate::client::get_client();
        let result = client.ugc().item_install_info(item_id);

        Ok(match result {
            Some(install_info) => Some(InstallInfo {
                folder: install_info.folder,
                size_on_disk: BigInt::from(install_info.size_on_disk),
                timestamp: install_info.timestamp,
            }),
            None => None,
        })
    }

    #[napi]
    pub fn download_info(item_id: BigInt) -> Result<Option<DownloadInfo>, Error> {
        let item_id = published_file_id(&item_id)?;
        let client = crate::client::get_client();
        let result = client.ugc().item_download_info(item_id);

        Ok(result.map(|download_info| DownloadInfo {
            current: BigInt::from(download_info.0),
            total: BigInt::from(download_info.1),
        }))
    }

    #[napi]
    pub fn download(item_id: BigInt, high_priority: bool) -> Result<bool, Error> {
        let item_id = published_file_id(&item_id)?;
        let client = crate::client::get_client();
        Ok(client.ugc().download_item(item_id, high_priority))
    }

    #[napi]
    pub fn suspend_downloads(suspend: bool) {
        crate::client::get_client().ugc().suspend_downloads(suspend);
    }

    #[napi]
    pub fn get_subscribed_items(
        include_locally_disabled: Option<bool>,
    ) -> Result<Vec<BigInt>, Error> {
        let _client = crate::client::get_client();
        let raw = unsafe { steamworks::sys::SteamAPI_SteamUGC_v021() };
        if raw.is_null() {
            return Err(Error::from_reason("Steam UGC is unavailable"));
        }
        let include = include_locally_disabled.unwrap_or(false);
        let count =
            unsafe { steamworks::sys::SteamAPI_ISteamUGC_GetNumSubscribedItems(raw, include) };
        if count > MAX_SUBSCRIBED_ITEMS {
            return Err(Error::from_reason(
                "Subscribed item count exceeds the 100000 item limit",
            ));
        }
        let mut items = vec![0u64; count as usize];
        let returned = unsafe {
            steamworks::sys::SteamAPI_ISteamUGC_GetSubscribedItems(
                raw,
                items.as_mut_ptr(),
                count,
                include,
            )
        };
        items.truncate(returned.min(count) as usize);
        Ok(items.into_iter().map(BigInt::from).collect())
    }

    #[napi]
    pub async fn start_playtime_tracking(item_ids: Vec<BigInt>) -> Result<(), Error> {
        if item_ids.is_empty() || item_ids.len() > MAX_PLAYTIME_ITEMS {
            return Err(Error::from_reason("Expected 1 to 100 workshop item IDs"));
        }
        let ids = item_ids
            .iter()
            .map(published_file_id)
            .collect::<Result<Vec<_>, _>>()?;
        let client = crate::client::get_client();
        let (tx, rx) = oneshot::channel();
        crate::api::runtime::without_dispatch(|| {
            client.ugc().start_playtime_tracking(&ids, move |result| {
                let _ = tx.send(result);
            });
        });
        tokio::time::timeout(std::time::Duration::from_secs(30), rx)
            .await
            .map_err(|_| Error::from_reason("Timed out waiting for playtime callback"))?
            .map_err(|_| Error::from_reason("Playtime callback dropped"))?
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn stop_playtime_tracking(item_ids: Vec<BigInt>) -> Result<(), Error> {
        if item_ids.is_empty() || item_ids.len() > MAX_PLAYTIME_ITEMS {
            return Err(Error::from_reason("Expected 1 to 100 workshop item IDs"));
        }
        let ids = item_ids
            .iter()
            .map(published_file_id)
            .collect::<Result<Vec<_>, _>>()?;
        let client = crate::client::get_client();
        let (tx, rx) = oneshot::channel();
        crate::api::runtime::without_dispatch(|| {
            client.ugc().stop_playtime_tracking(&ids, move |result| {
                let _ = tx.send(result);
            });
        });
        tokio::time::timeout(std::time::Duration::from_secs(30), rx)
            .await
            .map_err(|_| Error::from_reason("Timed out waiting for playtime callback"))?
            .map_err(|_| Error::from_reason("Playtime callback dropped"))?
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn stop_playtime_tracking_for_all_items() -> Result<(), Error> {
        let client = crate::client::get_client();
        let (tx, rx) = oneshot::channel();
        crate::api::runtime::without_dispatch(|| {
            client
                .ugc()
                .stop_playtime_tracking_for_all_items(move |result| {
                    let _ = tx.send(result);
                });
        });
        tokio::time::timeout(std::time::Duration::from_secs(30), rx)
            .await
            .map_err(|_| Error::from_reason("Timed out waiting for playtime callback"))?
            .map_err(|_| Error::from_reason("Playtime callback dropped"))?
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn delete_item(item_id: BigInt) -> Result<(), Error> {
        let item_id = published_file_id(&item_id)?;
        let client = crate::client::get_client();
        let (tx, rx) = oneshot::channel();

        crate::api::runtime::without_dispatch(|| {
            client.ugc().delete_item(item_id, move |result| {
                let _ = tx.send(result);
            });
        });

        let result = rx
            .await
            .map_err(|_| Error::from_reason("Delete item callback dropped"))?;
        match result {
            Ok(()) => Ok(()),
            Err(e) => Err(Error::from_reason(e.to_string())),
        }
    }
}
