#define SteamAPI_ISteamApps_BIsAppInstalled fixture_unused_is_installed
#define SteamAPI_ManualDispatch_GetNextCallback fixture_unused_next_callback
#define SteamAPI_ManualDispatch_FreeLastCallback fixture_unused_free_callback
#include "../deviation-native/sdk.c"
#undef SteamAPI_ISteamApps_BIsAppInstalled
#undef SteamAPI_ManualDispatch_GetNextCallback
#undef SteamAPI_ManualDispatch_FreeLastCallback

#pragma pack(push, 4)
struct persona { uint64_t id; int32_t flags; };
struct lobby_created { int32_t result; uint64_t lobby; };
struct lobby_enter { uint64_t lobby; uint32_t permissions; bool locked; uint32_t response; };
struct stats_received { uint64_t game; int32_t result; uint64_t user; };
struct stats_stored { uint64_t game; int32_t result; };
struct achievement { uint64_t game; bool group; char name[128]; uint32_t current, max; };
struct icon { uint64_t game; char name[128]; bool achieved; int32_t handle; };
struct download { uint32_t app; uint64_t file; int32_t result; };
struct validation { uint64_t peer; uint32_t response; uint64_t owner; };
struct delete_result { int32_t result; uint64_t file; };
struct call_complete { uint64_t call; int32_t callback; uint32_t size; };
struct server_failure { int32_t reason; bool retrying; };
struct chat_update { uint64_t lobby, changed, actor; uint32_t state; };
struct lobby_data { uint64_t lobby, member; uint8_t success; };
struct chat_message { uint64_t lobby, user; uint8_t type; uint32_t id; };
struct p2p_failure { uint64_t peer; uint8_t error; };
struct lobby_join { uint64_t lobby, friend_id; };
struct transaction { uint32_t app; uint64_t order; uint8_t authorized; };
#pragma pack(pop)
struct relay { int32_t availability, ping, config, any; char message[256]; };

static struct persona persona = { UINT64_C(76561198012345678), 1 };
static struct persona persona_multi = { UINT64_C(76561198012345678), 0x40000003 };
static struct persona persona_empty = { UINT64_C(76561198012345678), 0 };
static struct lobby_created lobby_created = { 1, UINT64_C(18446744073709551614) };
static struct lobby_enter lobby_enter = { UINT64_C(18446744073709551614), 0, true, 4 };
static struct stats_received stats_ok = { UINT64_C(18446744073709551614), 1, UINT64_C(18446744073709551614) };
static struct stats_received stats_error = { UINT64_C(18446744073709551614), 2, UINT64_C(18446744073709551614) };
static struct stats_stored stored = { UINT64_C(18446744073709551614), 1 };
static struct achievement achievement_event = { UINT64_C(18446744073709551614), false, "ACH_TEST", 3, 10 };
static struct icon icon_event = { UINT64_C(18446744073709551614), "ACH_TEST", true, 17 };
static struct download download_event = { 480, UINT64_C(18446744073709551614), 2 };
static struct validation validation_event = { UINT64_C(18446744073709551614), 6, UINT64_MAX };
static struct stats_stored global_stats = { UINT64_C(18446744073709551614), 2 };
static struct delete_result delete_failure = { 2, UINT64_C(18446744073709551614) };
static struct delete_result delete_success = { 1, UINT64_C(18446744073709551614) };
static struct call_complete delete_complete = { 333, 3417, sizeof(struct delete_result) };
static struct relay relay = { -102, 1, 2, 100, "ready" };
static struct relay relay_full = { -101, 0, -10, 2147483647, {0} };
static uint32_t remote = UINT32_MAX;
static int32_t server_disconnected = 2;
static struct server_failure server_failed = { 3, true };
static struct chat_update chat = { UINT64_C(18446744073709551614), UINT64_C(76561198012345678), 123, 8 };
static struct lobby_data lobby_data = { UINT64_C(18446744073709551614), UINT64_C(76561198012345678), 1 };
static struct chat_message chat_message = { UINT64_C(18446744073709551614), UINT64_C(76561198012345678), 1, 42 };
static uint64_t peer = UINT64_C(76561198012345678);
static struct p2p_failure p2p_failure = { UINT64_C(76561198012345678), 4 };
static struct lobby_join lobby_join = { UINT64_C(18446744073709551614), UINT64_C(76561198012345678) };
static struct transaction transaction = { 480, UINT64_C(18446744073709551614), 1 };
static uint8_t empty;
static uint32_t pending;
static bool delete_requested;

bool SteamAPI_ISteamApps_BIsAppInstalled(void *self, uint32_t which) {
    assert(self == &app_id && which >= 1 && which <= 30);
    pthread_mutex_lock(&lock);
    assert(!pending);
    assert(which != 18 || delete_requested);
    if (which == 17) {
        memset(relay_full.message, 'x', sizeof(relay_full.message));
        relay_full.message[1] = (char)0xff;
    }
    pending = which;
    pthread_mutex_unlock(&lock);
    return true;
}

bool SteamAPI_ManualDispatch_GetNextCallback(int32_t pipe, Callback *callback) {
    assert(pipe == 42);
    pthread_mutex_lock(&lock);
    switch (pending) {
        case 1: *callback = (Callback){1, 304, &persona, sizeof(persona)}; break;
        case 2: *callback = (Callback){1, 513, &lobby_created, sizeof(lobby_created)}; break;
        case 3: *callback = (Callback){1, 504, &lobby_enter, sizeof(lobby_enter)}; break;
        case 4: *callback = (Callback){1, 1014, &empty, 1}; break;
        case 5: *callback = (Callback){1, 1101, &stats_ok, sizeof(stats_ok)}; break;
        case 6: *callback = (Callback){1, 1101, &stats_error, sizeof(stats_error)}; break;
        case 7: *callback = (Callback){1, 1102, &stored, sizeof(stored)}; break;
        case 8: *callback = (Callback){1, 1103, &achievement_event, sizeof(achievement_event)}; break;
        case 9: *callback = (Callback){1, 1109, &icon_event, sizeof(icon_event)}; break;
        case 10: *callback = (Callback){1, 3406, &download_event, sizeof(download_event)}; break;
        case 11: *callback = (Callback){1, 5701, &remote, sizeof(remote)}; break;
        case 12: *callback = (Callback){1, 5702, &remote, sizeof(remote)}; break;
        case 13: *callback = (Callback){1, 143, &validation_event, sizeof(validation_event)}; break;
        case 14: *callback = (Callback){1, 1281, &relay, sizeof(relay)}; break;
        case 15: *callback = (Callback){1, 1112, &global_stats, sizeof(global_stats)}; break;
        case 16: *callback = (Callback){1, 3417, &delete_failure, sizeof(delete_failure)}; break;
        case 17: *callback = (Callback){1, 1281, &relay_full, sizeof(relay_full)}; break;
        case 18: *callback = (Callback){1, 703, &delete_complete, sizeof(delete_complete)}; break;
        case 19: *callback = (Callback){1, 101, &empty, 1}; break;
        case 20: *callback = (Callback){1, 103, &server_disconnected, sizeof(server_disconnected)}; break;
        case 21: *callback = (Callback){1, 102, &server_failed, sizeof(server_failed)}; break;
        case 22: *callback = (Callback){1, 506, &chat, sizeof(chat)}; break;
        case 23: *callback = (Callback){1, 304, &persona_multi, sizeof(persona_multi)}; break;
        case 24: *callback = (Callback){1, 304, &persona_empty, sizeof(persona_empty)}; break;
        case 25: *callback = (Callback){1, 505, &lobby_data, sizeof(lobby_data)}; break;
        case 26: *callback = (Callback){1, 507, &chat_message, sizeof(chat_message)}; break;
        case 27: *callback = (Callback){1, 1202, &peer, sizeof(peer)}; break;
        case 28: *callback = (Callback){1, 1203, &p2p_failure, sizeof(p2p_failure)}; break;
        case 29: *callback = (Callback){1, 333, &lobby_join, sizeof(lobby_join)}; break;
        case 30: *callback = (Callback){1, 152, &transaction, sizeof(transaction)}; break;
        default: pthread_mutex_unlock(&lock); return false;
    }
    pthread_mutex_unlock(&lock);
    return true;
}

void *SteamAPI_SteamUGC_v021(void) { return &app_id; }

uint64_t SteamAPI_ISteamUGC_DeleteItem(void *self, uint64_t file) {
    assert(self == &app_id && file == UINT64_C(18446744073709551614));
    pthread_mutex_lock(&lock);
    assert(!delete_requested);
    delete_requested = true;
    pthread_mutex_unlock(&lock);
    return 333;
}

bool SteamAPI_ManualDispatch_GetAPICallResult(int32_t pipe, uint64_t call, void *out,
                                             int32_t size, int32_t callback, bool *failed) {
    assert(pipe == 42 && call == 333 && size == sizeof(delete_success) && callback == 3417);
    memcpy(out, &delete_success, sizeof(delete_success));
    *failed = false;
    return true;
}

void SteamAPI_ManualDispatch_FreeLastCallback(int32_t pipe) {
    assert(pipe == 42);
    pthread_mutex_lock(&lock);
    assert(pending);
    pending = 0;
    pthread_mutex_unlock(&lock);
}
