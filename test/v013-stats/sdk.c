#define SteamAPI_ManualDispatch_GetNextCallback fixture_base_next
#define SteamAPI_ManualDispatch_FreeLastCallback fixture_base_free
#include "../deviation-native/sdk.c"
#undef SteamAPI_ManualDispatch_GetNextCallback
#undef SteamAPI_ManualDispatch_FreeLastCallback

#pragma pack(push, 4)
typedef struct { uint64_t handle; int32_t callback; uint32_t size; } CallCompleted;
typedef struct { uint64_t game_id; int32_t result; } GlobalStats;
typedef struct { uint64_t board; uint8_t found; } BoardFound;
typedef struct { uint64_t board, entries; int32_t count; } BoardDownloaded;
typedef struct { uint64_t user; int32_t rank, score, count; uint64_t ugc; } BoardEntry;
#pragma pack(pop)
static bool global_issued[6], global_delivered[6], global_current;
static int current_day;
static CallCompleted completed;
static CallCompleted board_pending;
static bool board_current;

uint64_t SteamAPI_ISteamUserStats_FindLeaderboard(void *self, const char *name) {
    assert(self == &app_id && !strcmp(name, "scores"));
    pthread_mutex_lock(&lock);
    board_pending = (CallCompleted){2001, 1104, sizeof(BoardFound)};
    pthread_mutex_unlock(&lock);
    return 2001;
}
uint64_t SteamAPI_ISteamUserStats_DownloadLeaderboardEntries(void *self, uint64_t board, int kind, int first, int last) {
    assert(self == &app_id && board == 71 && kind == 1 && first == -1 && last == 1);
    pthread_mutex_lock(&lock);
    board_pending = (CallCompleted){2002, 1105, sizeof(BoardDownloaded)};
    pthread_mutex_unlock(&lock);
    return 2002;
}
bool SteamAPI_ISteamUserStats_GetDownloadedLeaderboardEntry(void *self, uint64_t entries, int index,
    BoardEntry *entry, int32_t *details, int capacity) {
    assert(self == &app_id && entries == 72 && index == 0 && capacity == 2);
    *entry = (BoardEntry){steam_id, 3, 50, 4, 0};
    details[0] = -5; details[1] = 23;
    return true;
}

uint64_t SteamAPI_ISteamUserStats_RequestGlobalStats(void *self, int32_t days) {
    assert(self == &app_id && days >= 1 && days <= 5);
    pthread_mutex_lock(&lock);
    assert(!global_issued[days]);
    global_issued[days] = true;
    pthread_mutex_unlock(&lock);
    return 1000 + (uint64_t)days;
}
bool SteamAPI_ManualDispatch_GetNextCallback(int32_t pipe, Callback *callback) {
    assert(pipe == 42);
    pthread_mutex_lock(&lock);
    if (board_pending.handle) {
        completed = board_pending;
        board_current = true;
        *callback = (Callback){1, 703, &completed, sizeof(completed)};
        pthread_mutex_unlock(&lock);
        return true;
    }
    int day = 0;
    if (global_issued[1] && global_issued[2] && !global_delivered[2]) day = 2;
    else if (global_delivered[2] && !global_delivered[1]) day = 1;
    else if (global_issued[3] && !global_delivered[3]) day = 3;
    else if (global_issued[4] && !global_delivered[4]) day = 4;
    if (day) {
        current_day = day;
        global_current = true;
        completed = (CallCompleted){1000 + (uint64_t)day, 1112, sizeof(GlobalStats)};
        *callback = (Callback){1, 703, &completed, sizeof(completed)};
        pthread_mutex_unlock(&lock);
        return true;
    }
    pthread_mutex_unlock(&lock);
    return fixture_base_next(pipe, callback);
}
void SteamAPI_ManualDispatch_FreeLastCallback(int32_t pipe) {
    pthread_mutex_lock(&lock);
    if (board_current) {
        board_pending.handle = 0;
        board_current = false;
        pthread_mutex_unlock(&lock);
        return;
    }
    bool global = global_current;
    if (global) {
        global_delivered[current_day] = true;
        global_current = false;
    }
    pthread_mutex_unlock(&lock);
    if (!global) fixture_base_free(pipe);
}
bool SteamAPI_ManualDispatch_GetAPICallResult(int32_t pipe, uint64_t handle, void *out,
    int32_t size, int32_t expected, bool *failed) {
    if (handle == 2001) {
        assert(pipe == 42 && size == sizeof(BoardFound) && expected == 1104);
        BoardFound found = {71, 1};
        memcpy(out, &found, sizeof(found)); *failed = false; return true;
    }
    if (handle == 2002) {
        assert(pipe == 42 && size == sizeof(BoardDownloaded) && expected == 1105);
        BoardDownloaded download = {71, 72, 1};
        memcpy(out, &download, sizeof(download)); *failed = false; return true;
    }
    assert(pipe == 42 && handle >= 1001 && handle <= 1004 && expected == 1112 && size == sizeof(GlobalStats));
    int day = (int)(handle - 1000);
    GlobalStats result = {UINT64_C(9007199254740994) + (uint64_t)day, day == 4 ? 2 : 1};
    memcpy(out, &result, sizeof(result));
    *failed = day == 3;
    return true;
}

void *SteamAPI_SteamTimeline_v004(void) { return &app_id; }
void SteamAPI_ISteamTimeline_SetTimelineGameMode(void *self, int mode) {
    assert(self == &app_id && mode == 2);
}
void SteamAPI_ISteamTimeline_SetTimelineTooltip(void *self, const char *description, float seconds) {
    assert(self == &app_id && strcmp(description, "state") == 0 && seconds == 2.5f);
}
void SteamAPI_ISteamTimeline_ClearTimelineTooltip(void *self, float seconds) {
    assert(self == &app_id && seconds == 3.0f);
}
uint64_t SteamAPI_ISteamTimeline_AddRangeTimelineEvent(void *self, const char *title,
    const char *description, const char *icon, uint32_t priority, float start, float duration, int clip) {
    assert(self == &app_id && strcmp(title, "title") == 0 && strcmp(description, "description") == 0);
    assert(strcmp(icon, "icon") == 0 && priority == 7 && start == -2.0f && duration == 4.0f && clip == 3);
    return 1;
}
bool SteamAPI_ISteamUserStats_GetGlobalStatInt64(void *self, const char *name, int64_t *value) {
    assert(self == &app_id);
    if (strcmp(name, "missing") == 0) return false;
    *value = INT64_C(-9007199254740993); return true;
}
bool SteamAPI_ISteamUserStats_GetGlobalStatDouble(void *self, const char *name, double *value) {
    assert(self == &app_id);
    if (strcmp(name, "missing") == 0) return false;
    *value = 1.25; return true;
}
int32_t SteamAPI_ISteamUserStats_GetGlobalStatHistoryInt64(void *self, const char *name, int64_t *values, uint32_t size) {
    assert(self == &app_id && size == 16);
    if (strcmp(name, "missing") == 0) return -1;
    values[0] = INT64_C(-9007199254740993); values[1] = INT64_C(9007199254740993); return 2;
}
int32_t SteamAPI_ISteamUserStats_GetGlobalStatHistoryDouble(void *self, const char *name, double *values, uint32_t size) {
    assert(self == &app_id && size == 16);
    if (strcmp(name, "missing") == 0) return -1;
    values[0] = 1.25; values[1] = -3.5; return 2;
}
bool SteamAPI_ISteamUserStats_GetAchievementAndUnlockTime(void *self, const char *name, bool *achieved, uint32_t *time) {
    assert(self == &app_id);
    if (strcmp(name, "missing") == 0) return false;
    *achieved = true; *time = 123456; return true;
}
uint64_t SteamAPI_ISteamUserStats_RequestUserStats(void *self, uint64_t id) {
    assert(self == &app_id && (id == steam_id || id == steam_id + 1)); return id == steam_id ? 123 : 0;
}
bool SteamAPI_ISteamUserStats_GetUserStatInt32(void *self, uint64_t id, const char *name, int32_t *value) {
    assert(self == &app_id && id == steam_id);
    if (strcmp(name, "missing") == 0) return false;
    *value = -42; return true;
}
bool SteamAPI_ISteamUserStats_GetUserStatFloat(void *self, uint64_t id, const char *name, float *value) {
    assert(self == &app_id && id == steam_id);
    if (strcmp(name, "missing") == 0) return false;
    *value = 1.25f; return true;
}
bool SteamAPI_ISteamUserStats_GetUserAchievement(void *self, uint64_t id, const char *name, bool *value) {
    assert(self == &app_id && id == steam_id);
    if (strcmp(name, "missing") == 0) return false;
    *value = strcmp(name, "unlocked") == 0; return true;
}
int32_t SteamAPI_ISteamUserStats_GetAchievementIcon(void *self, const char *name) {
    assert(self == &app_id);
    if (strcmp(name, "missing") == 0) return 0;
    if (strcmp(name, "large") == 0) return 2;
    if (strcmp(name, "size-error") == 0) return 3;
    if (strcmp(name, "rgba-error") == 0) return 4;
    return 1;
}
bool SteamAPI_ISteamUtils_GetImageSize(void *self, int32_t image, uint32_t *width, uint32_t *height) {
    assert(self == &app_id);
    if (image == 3) return false;
    *width = image == 2 ? UINT32_MAX : 2; *height = 2; return true;
}
bool SteamAPI_ISteamUtils_GetImageRGBA(void *self, int32_t image, uint8_t *data, int32_t size) {
    assert(self == &app_id && (image == 1 || image == 4) && size == 16);
    if (image == 4) return false;
    for (int i = 0; i < size; i++) data[i] = (uint8_t)i;
    return true;
}
int SteamAPI_ISteamUser_UserHasLicenseForApp(void *self, uint64_t id, uint32_t app) {
    assert(self == &app_id && id == steam_id); return app == 1 ? 0 : app == 2 ? 1 : 2;
}
const char *SteamAPI_ISteamApps_GetLaunchQueryParam(void *self, const char *key) {
    assert(self == &app_id);
    return strcmp(key, "token") == 0 ? "fixture value" : "";
}
void SteamAPI_ISteamFriends_ActivateGameOverlayInviteDialogConnectString(void *self, const char *connect) {
    assert(self == &app_id && strcmp(connect, "connect") == 0);
}
