#define SteamAPI_ManualDispatch_GetNextCallback base_GetNextCallback
#define SteamAPI_ManualDispatch_FreeLastCallback base_FreeLastCallback
#include "../deviation-native/sdk.c"
#undef SteamAPI_ManualDispatch_GetNextCallback
#undef SteamAPI_ManualDispatch_FreeLastCallback

#pragma pack(push, 4)
typedef struct { uint64_t handle; int32_t id; uint32_t size; } Completed;
#pragma pack(pop)
static Completed completed;
static uint8_t result_data[300];
static bool pending;
static uint64_t handle = 100;
static uint32_t mask;
static uint32_t server_ip;
static uint16_t server_port;
static uint64_t server_id;
static unsigned server_reads;
static bool language_set;
static bool query_ids, query_total;
static bool query_error;
static bool query_search, query_cloud_filter;
static unsigned query_kind, all_queries, user_queries;
static unsigned failed_query_releases;

static bool timeout_case(void) {
    const char *value = getenv("FIXTURE_CASE");
    return value && strncmp(value, "content-timeout", 15) == 0;
}
static unsigned filter_flags;

static uint64_t issue(int id, int size, int status) {
    pthread_mutex_lock(&lock);
    assert(!pending);
    completed = (Completed){++handle, id, size};
    memset(result_data, 0, sizeof(result_data));
    memcpy(result_data, &status, 4);
    pending = true;
    pthread_mutex_unlock(&lock);
    return handle;
}

bool SteamAPI_ManualDispatch_GetNextCallback(int32_t pipe, Callback *callback) {
    assert(pipe == 42);
    if (timeout_case()) return false;
    pthread_mutex_lock(&lock);
    if (!pending) { pthread_mutex_unlock(&lock); return false; }
    *callback = (Callback){1, 703, &completed, sizeof(completed)};
    pthread_mutex_unlock(&lock);
    return true;
}
void SteamAPI_ManualDispatch_FreeLastCallback(int32_t pipe) {
    assert(pipe == 42);
    pthread_mutex_lock(&lock);
    assert(pending);
    pending = false;
    pthread_mutex_unlock(&lock);
}
bool SteamAPI_ManualDispatch_GetAPICallResult(int32_t pipe, uint64_t call, void *out, int32_t size, int32_t id, bool *failed) {
    assert(pipe == 42 && pending && call == completed.handle && size == (int32_t)completed.size && id == completed.id);
    memcpy(out, result_data, size);
    *failed = false;
    return true;
}
void *SteamAPI_SteamUGC_v021(void) { return &app_id; }
void *SteamAPI_SteamRemoteStorage_v016(void) { return &app_id; }
void *SteamAPI_SteamMatchmaking_v009(void) { return &app_id; }
uint32_t SteamAPI_ISteamUGC_GetNumSubscribedItems(void *self, bool include) {
    assert(self == &app_id);
    if (getenv("CONTENT_OVERSIZED_SUBSCRIPTIONS")) return 100001;
    return include ? 3 : 2;
}
uint32_t SteamAPI_ISteamUGC_GetSubscribedItems(void *self, uint64_t *ids, uint32_t count, bool include) {
    assert(self == &app_id && count == (include ? 3U : 2U));
    ids[0] = UINT64_C(18446744073709551614);
    ids[1] = 42;
    return 2;
}
uint64_t SteamAPI_ISteamUGC_StartPlaytimeTracking(void *self, uint64_t *ids, uint32_t count) {
    assert(self == &app_id);
    if (count == 100) {
        for (uint32_t i = 0; i < count; i++) assert(ids[i] == 42);
    } else {
        assert(count == 2 && ids[0] == UINT64_C(18446744073709551614) && ids[1] == 42);
    }
    return issue(3410, 4, 1);
}
uint64_t SteamAPI_ISteamUGC_StopPlaytimeTracking(void *self, uint64_t *ids, uint32_t count) {
    assert(self == &app_id && count == 1 && ids[0] == 42);
    return issue(3411, 4, 2);
}
uint64_t SteamAPI_ISteamUGC_StopPlaytimeTrackingForAllItems(void *self) {
    assert(self == &app_id);
    return issue(3411, 4, 1);
}
uint64_t SteamAPI_ISteamUGC_DeleteItem(void *self, uint64_t id) {
    assert(self == &app_id && id == 42);
    return issue(3417, 12, 2);
}
void SteamAPI_ISteamUGC_SuspendDownloads(void *self, bool suspend) {
    assert(self == &app_id && suspend);
}
uint64_t SteamAPI_ISteamUGC_StartItemUpdate(void *self, uint32_t id, uint64_t file) {
    assert(!getenv("CONTENT_FAIL_ON_UPDATE_START"));
    assert(self == &app_id && id == app_id && file == 42);
    return 777;
}
bool SteamAPI_ISteamUGC_SetItemUpdateLanguage(void *self, uint64_t update, const char *language) {
    assert(self == &app_id && update == 777);
    if (strcmp(language, "reject") == 0) return false;
    assert(strcmp(language, "german") == 0);
    language_set = true;
    return true;
}
uint64_t SteamAPI_ISteamUGC_SubmitItemUpdate(void *self, uint64_t update, const char *note) {
    assert(self == &app_id && update == 777 && language_set && note == NULL);
    uint64_t call = issue(3404, 16, 1);
    uint64_t file = 42;
    memcpy(result_data + 8, &file, 8);
    nanosleep(&(struct timespec){.tv_nsec = 100000000}, NULL);
    return call;
}
uint64_t SteamAPI_ISteamUGC_CreateItem(void *self, uint32_t id, int type) {
    assert(self == &app_id && id == app_id && type == 0);
    uint64_t call = issue(3403, 16, 1), file = 42;
    memcpy(result_data + 4, &file, 8);
    nanosleep(&(struct timespec){.tv_nsec = 100000000}, NULL);
    return call;
}
uint32_t SteamAPI_ISteamRemoteStorage_GetSyncPlatforms(void *self, const char *name) {
    assert(self == &app_id && strcmp(name, "save") == 0);
    return mask;
}
bool SteamAPI_ISteamRemoteStorage_SetSyncPlatforms(void *self, const char *name, uint32_t value) {
    assert(self == &app_id && strcmp(name, "save") == 0);
    mask = value;
    return true;
}
bool SteamAPI_ISteamRemoteStorage_FileForget(void *self, const char *name) {
    assert(self == &app_id && strcmp(name, "save") == 0);
    return true;
}
uint64_t SteamAPI_ISteamRemoteStorage_FileShare(void *self, const char *name) {
    assert(self == &app_id && strcmp(name, "save") == 0);
    uint64_t call = issue(1307, 272, 1);
    uint64_t value = UINT64_C(18446744073709551614);
    memcpy(result_data + 4, &value, 8);
    return call;
}
uint64_t SteamAPI_ISteamMatchmaking_CreateLobby(void *self, int type, int max) {
    assert(self == &app_id && type == 2 && max == 4);
    uint64_t call = issue(513, 12, 1);
    uint64_t lobby = 99;
    memcpy(result_data + 4, &lobby, 8);
    return call;
}
void SteamAPI_ISteamMatchmaking_SetLobbyGameServer(void *self, uint64_t lobby, uint32_t ip, uint16_t port, uint64_t id) {
    assert(self == &app_id && lobby == 99);
    server_ip = ip;
    server_port = port;
    server_id = id;
}
bool SteamAPI_ISteamMatchmaking_GetLobbyGameServer(void *self, uint64_t lobby, uint32_t *ip, uint16_t *port, uint64_t *id) {
    assert(self == &app_id && lobby == (server_reads++ == 0 ? 99 : 100));
    *ip = server_ip;
    *port = server_port;
    *id = server_id;
    return true;
}
void SteamAPI_ISteamMatchmaking_AddRequestLobbyListStringFilter(void *self, const char *key, const char *value, int cmp) {
    assert(self == &app_id && !strcmp(key, "map") && !strcmp(value, "arena") && cmp == 0);
    filter_flags |= 1;
}
void SteamAPI_ISteamMatchmaking_AddRequestLobbyListNumericalFilter(void *self, const char *key, int value, int cmp) {
    assert(self == &app_id && !strcmp(key, "rank") && value == 3 && cmp == 1);
    filter_flags |= 2;
}
void SteamAPI_ISteamMatchmaking_AddRequestLobbyListNearValueFilter(void *self, const char *key, int value) {
    assert(self == &app_id && !strcmp(key, "rank") && value == 4);
    filter_flags |= 4;
}
void SteamAPI_ISteamMatchmaking_AddRequestLobbyListResultCountFilter(void *self, int count) {
    assert(self == &app_id && count == 10);
    filter_flags |= 8;
}
uint64_t SteamAPI_ISteamMatchmaking_RequestLobbyList(void *self) {
    assert(self == &app_id && filter_flags == 15);
    return issue(510, 4, 1);
}
uint64_t SteamAPI_ISteamMatchmaking_GetLobbyByIndex(void *self, int index) {
    assert(self == &app_id && index == 0);
    return 99;
}
uint64_t SteamAPI_ISteamUGC_CreateQueryAllUGCRequestPage(void *self, int query, int type, uint32_t creator, uint32_t consumer, uint32_t page) {
    assert(self == &app_id && query == 0 && type == 0 && (creator == 480 || creator == 481) && consumer == 480 && page == 1);
    query_ids = query_total = false;
    query_error = creator == 481;
    query_search = query_cloud_filter = false;
    query_kind = 1;
    return 111;
}
uint64_t SteamAPI_ISteamUGC_CreateQueryUserUGCRequest(void *self, uint32_t account, int list, int type, int sort, uint32_t creator, uint32_t consumer, uint32_t page) {
    assert(self == &app_id && account == 1 && list == 0 && type == 0 && sort == 1 && creator == 0 && consumer == 0 && page == 1);
    query_ids = query_total = query_error = query_search = query_cloud_filter = false;
    query_kind = 2;
    return 111;
}
uint64_t SteamAPI_ISteamUGC_CreateQueryUGCDetailsRequest(void *self, uint64_t *ids, uint32_t count) {
    assert(self == &app_id && count == 1 && ids[0] == 42);
    query_ids = query_total = query_error = query_search = query_cloud_filter = false;
    query_kind = 3;
    return 111;
}
bool SteamAPI_ISteamUGC_SetSearchText(void *self, uint64_t query, const char *text) {
    assert(self == &app_id && query == 111 && !strcmp(text, "puzzle"));
    query_search = true;
    return true;
}
bool SteamAPI_ISteamUGC_SetCloudFileNameFilter(void *self, uint64_t query, const char *name) {
    assert(self == &app_id && query == 111 && !strcmp(name, "save.dat"));
    query_cloud_filter = true;
    return true;
}
bool SteamAPI_ISteamUGC_SetReturnOnlyIDs(void *self, uint64_t query, bool value) {
    assert(self == &app_id && query == 111 && value);
    query_ids = true;
    return true;
}
bool SteamAPI_ISteamUGC_SetReturnTotalOnly(void *self, uint64_t query, bool value) {
    assert(self == &app_id && query == 111 && value);
    query_total = true;
    return true;
}
uint64_t SteamAPI_ISteamUGC_SendQueryUGCRequest(void *self, uint64_t query) {
    assert(self == &app_id && query == 111 && !(query_ids && query_total));
    if (query_kind == 1 && all_queries++ == 0) assert(query_search);
    if (query_kind == 2) { user_queries++; assert(query_cloud_filter); }
    uint64_t call = issue(3401, 280, 0);
    uint32_t status = query_error ? 2 : 1, returned = query_total ? 0 : 1, total = 9;
    memcpy(result_data + 8, &status, 4);
    memcpy(result_data + 12, &returned, 4);
    memcpy(result_data + 16, &total, 4);
    nanosleep(&(struct timespec){.tv_nsec = 100000000}, NULL);
    return call;
}
bool SteamAPI_ISteamUGC_GetQueryUGCResult(void *self, uint64_t query, uint32_t index, void *details) {
    assert(self == &app_id && query == 111 && index == 0);
    memset(details, 0, 9772);
    uint64_t id = UINT64_C(18446744073709551614);
    uint32_t status = 1;
    memcpy(details, &id, 8);
    memcpy((uint8_t *)details + 8, &status, 4);
    if (!query_ids) memcpy((uint8_t *)details + 24, "Fixture title", 14);
    return true;
}
bool SteamAPI_ISteamUGC_GetQueryUGCPreviewURL(void *self, uint64_t query, uint32_t index, char *url, uint32_t size) {
    assert(self == &app_id && query == 111 && index == 0 && !query_ids && url && size);
    return false;
}
bool SteamAPI_ISteamUGC_GetQueryUGCStatistic(void *self, uint64_t query, uint32_t index, int statistic, uint64_t *value) {
    assert(self == &app_id && query == 111 && index == 0 && !query_ids && statistic >= 0);
    *value = 7;
    return true;
}
bool SteamAPI_ISteamUGC_ReleaseQueryUGCRequest(void *self, uint64_t query) {
    assert(self == &app_id && query == 111);
    if (query_error) failed_query_releases++;
    return true;
}
__attribute__((destructor)) static void verify_failed_query_cleanup(void) {
    if (!timeout_case()) {
        assert(failed_query_releases == 2);
        fprintf(stderr, "fixture released 2 failed queries\n");
    }
}
