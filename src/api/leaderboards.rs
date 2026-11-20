use napi_derive::napi;

#[napi]
pub mod leaderboards {
    use napi::bindgen_prelude::{BigInt, Error};
    use steamworks::{Client, Leaderboard, SteamError};
    use tokio::sync::oneshot;

    const MAX_DETAILS: usize = 64;
    #[derive(Debug)]
    #[napi]
    pub enum LeaderboardSortMethod {
        Ascending,
        Descending,
    }

    impl From<LeaderboardSortMethod> for steamworks::LeaderboardSortMethod {
        fn from(val: LeaderboardSortMethod) -> Self {
            match val {
                LeaderboardSortMethod::Ascending => steamworks::LeaderboardSortMethod::Ascending,
                LeaderboardSortMethod::Descending => steamworks::LeaderboardSortMethod::Descending,
            }
        }
    }

    #[derive(Debug)]
    #[napi]
    pub enum LeaderboardDisplayType {
        Numeric,
        TimeSeconds,
        TimeMilliSeconds,
    }

    impl From<LeaderboardDisplayType> for steamworks::LeaderboardDisplayType {
        fn from(val: LeaderboardDisplayType) -> Self {
            match val {
                LeaderboardDisplayType::Numeric => steamworks::LeaderboardDisplayType::Numeric,
                LeaderboardDisplayType::TimeSeconds => {
                    steamworks::LeaderboardDisplayType::TimeSeconds
                }
                LeaderboardDisplayType::TimeMilliSeconds => {
                    steamworks::LeaderboardDisplayType::TimeMilliSeconds
                }
            }
        }
    }

    #[derive(Debug)]
    #[napi]
    pub enum LeaderboardDataRequest {
        Global,
        GlobalAroundUser,
        Friends,
    }

    impl From<LeaderboardDataRequest> for steamworks::LeaderboardDataRequest {
        fn from(val: LeaderboardDataRequest) -> Self {
            match val {
                LeaderboardDataRequest::Global => steamworks::LeaderboardDataRequest::Global,
                LeaderboardDataRequest::GlobalAroundUser => {
                    steamworks::LeaderboardDataRequest::GlobalAroundUser
                }
                LeaderboardDataRequest::Friends => steamworks::LeaderboardDataRequest::Friends,
            }
        }
    }

    #[derive(Debug)]
    #[napi]
    pub enum UploadScoreMethod {
        KeepBest,
        ForceUpdate,
    }

    impl From<UploadScoreMethod> for steamworks::UploadScoreMethod {
        fn from(val: UploadScoreMethod) -> Self {
            match val {
                UploadScoreMethod::KeepBest => steamworks::UploadScoreMethod::KeepBest,
                UploadScoreMethod::ForceUpdate => steamworks::UploadScoreMethod::ForceUpdate,
            }
        }
    }

    #[napi(object)]
    pub struct LeaderboardEntry {
        pub steam_id: BigInt,
        pub global_rank: i32,
        pub score: i32,
        pub details: Vec<i32>,
    }

    #[napi(object)]
    pub struct LeaderboardScoreUploaded {
        pub score: i32,
        pub score_changed: bool,
        pub global_rank_new: i32,
        pub global_rank_previous: i32,
    }

    #[napi(object)]
    pub struct LeaderboardInfo {
        pub name: String,
        pub entry_count: i32,
        pub sort_method: Option<LeaderboardSortMethod>,
        pub display_type: Option<LeaderboardDisplayType>,
    }

    async fn callback<T>(rx: oneshot::Receiver<Result<T, SteamError>>) -> Result<T, Error> {
        tokio::time::timeout(std::time::Duration::from_secs(30), rx)
            .await
            .map_err(|_| Error::from_reason("Steam leaderboard callback timed out"))?
            .map_err(|_| Error::from_reason("Steam leaderboard callback was dropped"))?
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    fn validate_name(name: &str) -> Result<(), Error> {
        if name.is_empty() || name.len() > 128 || name.contains('\0') {
            return Err(Error::from_reason(
                "Leaderboard name must contain 1 to 128 UTF-8 bytes and no NUL",
            ));
        }
        Ok(())
    }

    async fn lookup(
        client: &Client,
        name: &str,
        create: Option<(LeaderboardSortMethod, LeaderboardDisplayType)>,
    ) -> Result<Option<Leaderboard>, Error> {
        let (tx, rx) = oneshot::channel();
        let send = move |result| {
            let _ = tx.send(result);
        };
        match create {
            Some((sort, display)) => client.user_stats().find_or_create_leaderboard(
                name,
                sort.into(),
                display.into(),
                send,
            ),
            None => client.user_stats().find_leaderboard(name, send),
        }
        callback(rx).await
    }

    async fn existing(client: &Client, name: &str) -> Result<Leaderboard, Error> {
        lookup(client, name, None)
            .await?
            .ok_or_else(|| Error::from_reason("Leaderboard not found"))
    }

    #[napi]
    pub async fn find_leaderboard(name: String) -> Result<bool, Error> {
        validate_name(&name)?;
        Ok(lookup(&crate::client::get_client(), &name, None)
            .await?
            .is_some())
    }

    #[napi]
    pub async fn find_or_create_leaderboard(
        name: String,
        sort_method: LeaderboardSortMethod,
        display_type: LeaderboardDisplayType,
    ) -> Result<bool, Error> {
        validate_name(&name)?;
        Ok(lookup(
            &crate::client::get_client(),
            &name,
            Some((sort_method, display_type)),
        )
        .await?
        .is_some())
    }

    #[napi]
    pub async fn upload_leaderboard_score(
        name: String,
        sort_method: LeaderboardSortMethod,
        display_type: LeaderboardDisplayType,
        method: UploadScoreMethod,
        score: i32,
        details: Option<Vec<i32>>,
    ) -> Result<LeaderboardScoreUploaded, Error> {
        validate_name(&name)?;
        let details = details.unwrap_or_default();
        if details.len() > MAX_DETAILS {
            return Err(Error::from_reason(
                "Leaderboard scores support at most 64 details",
            ));
        }
        let client = crate::client::get_client();
        let board = lookup(&client, &name, Some((sort_method, display_type)))
            .await?
            .ok_or_else(|| Error::from_reason("Could not find or create leaderboard"))?;
        let (tx, rx) = oneshot::channel();
        client.user_stats().upload_leaderboard_score(
            &board,
            method.into(),
            score,
            &details,
            move |result| {
                let _ = tx.send(result);
            },
        );
        let uploaded = callback(rx)
            .await?
            .ok_or_else(|| Error::from_reason("Steam rejected leaderboard score upload"))?;
        Ok(LeaderboardScoreUploaded {
            score: uploaded.score,
            score_changed: uploaded.was_changed,
            global_rank_new: uploaded.global_rank_new,
            global_rank_previous: uploaded.global_rank_previous,
        })
    }

    #[napi]
    pub async fn download_leaderboard_entries(
        name: String,
        request: LeaderboardDataRequest,
        range_start: i32,
        range_end: i32,
        max_details: Option<u32>,
    ) -> Result<Vec<LeaderboardEntry>, Error> {
        validate_name(&name)?;
        if !matches!(request, LeaderboardDataRequest::Friends) && range_start > range_end {
            return Err(Error::from_reason("Leaderboard range start exceeds end"));
        }
        if matches!(request, LeaderboardDataRequest::Global) && range_start < 1 {
            return Err(Error::from_reason("Global leaderboard ranks start at 1"));
        }
        let limit = max_details.unwrap_or(MAX_DETAILS as u32) as usize;
        if limit > MAX_DETAILS {
            return Err(Error::from_reason("maxDetails must be between 0 and 64"));
        }
        let client = crate::client::get_client();
        let board = existing(&client, &name).await?;
        let (tx, rx) = oneshot::channel();
        client.user_stats().download_leaderboard_entries(
            &board,
            request.into(),
            range_start as usize,
            range_end as usize,
            MAX_DETAILS,
            move |result| {
                let _ = tx.send(result);
            },
        );
        Ok(callback(rx)
            .await?
            .into_iter()
            .map(|mut e| {
                e.details.truncate(limit);
                LeaderboardEntry {
                    steam_id: e.user.raw().into(),
                    global_rank: e.global_rank,
                    score: e.score,
                    details: e.details,
                }
            })
            .collect())
    }

    #[napi]
    pub async fn get_leaderboard_entry_count(name: String) -> Result<i32, Error> {
        validate_name(&name)?;
        let client = crate::client::get_client();
        let board = existing(&client, &name).await?;
        Ok(client.user_stats().get_leaderboard_entry_count(&board))
    }

    #[napi]
    pub async fn get_leaderboard_info(name: String) -> Result<LeaderboardInfo, Error> {
        validate_name(&name)?;
        let client = crate::client::get_client();
        let board = existing(&client, &name).await?;
        let stats = client.user_stats();
        Ok(LeaderboardInfo {
            name: stats.get_leaderboard_name(&board),
            entry_count: stats.get_leaderboard_entry_count(&board),
            sort_method: stats
                .get_leaderboard_sort_method(&board)
                .map(|sort| match sort {
                    steamworks::LeaderboardSortMethod::Ascending => {
                        LeaderboardSortMethod::Ascending
                    }
                    steamworks::LeaderboardSortMethod::Descending => {
                        LeaderboardSortMethod::Descending
                    }
                }),
            display_type: stats
                .get_leaderboard_display_type(&board)
                .map(|display| match display {
                    steamworks::LeaderboardDisplayType::Numeric => LeaderboardDisplayType::Numeric,
                    steamworks::LeaderboardDisplayType::TimeSeconds => {
                        LeaderboardDisplayType::TimeSeconds
                    }
                    steamworks::LeaderboardDisplayType::TimeMilliSeconds => {
                        LeaderboardDisplayType::TimeMilliSeconds
                    }
                }),
        })
    }
}
