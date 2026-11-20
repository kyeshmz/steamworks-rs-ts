use napi_derive::napi;

#[napi]
pub mod achievement {
    use napi::bindgen_prelude::{Buffer, Error};
    use tokio::sync::oneshot;

    #[napi]
    pub fn activate(achievement: String) -> bool {
        if achievement.contains('\0') {
            return false;
        }
        let client = crate::client::get_client();
        client
            .user_stats()
            .achievement(&achievement)
            .set()
            .and_then(|_| client.user_stats().store_stats())
            .is_ok()
    }

    #[napi]
    pub fn is_activated(achievement: String) -> bool {
        if achievement.contains('\0') {
            return false;
        }
        let client = crate::client::get_client();
        client
            .user_stats()
            .achievement(&achievement)
            .get()
            .unwrap_or(false)
    }

    #[napi]
    pub fn clear(achievement: String) -> bool {
        if achievement.contains('\0') {
            return false;
        }
        let client = crate::client::get_client();
        client
            .user_stats()
            .achievement(&achievement)
            .clear()
            .and_then(|_| client.user_stats().store_stats())
            .is_ok()
    }

    #[napi]
    pub fn names() -> Vec<String> {
        let stats = crate::client::get_client().user_stats();
        if stats.get_num_achievements().is_err() {
            return Vec::new();
        }
        stats.get_achievement_names().unwrap_or_default()
    }

    #[napi]
    pub fn count() -> u32 {
        crate::client::get_client()
            .user_stats()
            .get_num_achievements()
            .unwrap_or_default()
    }

    #[napi]
    pub fn get_display_attribute(achievement: String, key: String) -> Option<String> {
        if achievement.contains('\0') || !matches!(key.as_str(), "name" | "desc" | "hidden") {
            return None;
        }
        crate::client::get_client()
            .user_stats()
            .achievement(&achievement)
            .get_achievement_display_attribute(&key)
            .ok()
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
    }

    #[napi]
    pub fn get_icon(achievement: String) -> Option<Buffer> {
        if achievement.contains('\0') {
            return None;
        }
        crate::client::get_client()
            .user_stats()
            .achievement(&achievement)
            .get_achievement_icon()
            .map(Buffer::from)
    }

    #[napi]
    pub async fn request_global_percentages() -> napi::Result<()> {
        let (tx, rx) = oneshot::channel();
        crate::client::get_client()
            .user_stats()
            .request_global_achievement_percentages(move |result| {
                let _ = tx.send(result);
            });
        tokio::time::timeout(std::time::Duration::from_secs(30), rx)
            .await
            .map_err(|_| {
                Error::from_reason("Timed out waiting for global achievement percentages")
            })?
            .map_err(|_| Error::from_reason("Global achievement percentage request was cancelled"))?
            .map(|_| ())
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub fn get_achieved_percent(achievement: String) -> Option<f64> {
        if achievement.contains('\0') {
            return None;
        }
        crate::client::get_client()
            .user_stats()
            .achievement(&achievement)
            .get_achievement_achieved_percent()
            .ok()
            .map(f64::from)
    }
}
