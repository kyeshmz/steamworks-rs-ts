#include <assert.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdatomic.h>
#include <string.h>

static atomic_int init_calls, submissions, drops;
static int modes[32], kind[32], polls[32];
static bool destroyed[32];
static const uint64_t item_id = UINT64_MAX - 1;

int SteamAPI_InitFlat(char *message) {
    int n = atomic_fetch_add(&init_calls, 1);
    assert(n <= 1);
    if (n == 1) return 0;
    strcpy(message, "fixture first init failure");
    return 1;
}
void SteamAPI_ManualDispatch_Init(void) {}
int SteamAPI_GetHSteamPipe(void) { return 1; }
void SteamAPI_ManualDispatch_RunFrame(int pipe) {}
#pragma pack(push, 4)
struct validation { uint64_t peer; uint32_t response; uint64_t owner; };
struct overlay_activation { uint8_t active; bool user_initiated; uint32_t app_id, pid; };
struct order_event { uint32_t app; uint64_t order; uint8_t authorized; };
struct callback_msg { int user, id; void *data; int size; };
#pragma pack(pop)
static const uint64_t peer = 76561197960265729ULL;
static struct validation validation;
static _Thread_local bool auth_thread;
static int accepted, rejected, ended, delivered, freed;
static bool active;
static atomic_int keyboard_event;
static atomic_bool hold_keyboard, release_keyboard;
static _Thread_local bool dispatching_keyboard;
static struct { bool submitted; uint32_t length, app; } dismissed = { true, 4, 480 };

bool SteamAPI_ISteamUtils_ShowGamepadTextInput(void *p, int mode, int lines, const char *description, uint32_t max, const char *existing) {
    assert(p == &init_calls && mode == 0 && lines == 0 && max == 20 && !existing);
    if (!strcmp(description, "unavailable")) return false;
    atomic_store(&hold_keyboard, !strcmp(description, "held"));
    dismissed.submitted = strcmp(description, "cancel") != 0;
    assert(atomic_exchange(&keyboard_event, 714) == 0);
    return true;
}
bool SteamAPI_ISteamUtils_GetEnteredGamepadTextInput(void *p, char *out, uint32_t capacity) {
    assert(dispatching_keyboard && dismissed.submitted && capacity >= 5);
    memcpy(out, "Caf!", 5);
    return true;
}
bool SteamAPI_ISteamUtils_ShowFloatingGamepadTextInput(void *p, int mode, int x, int y, int width, int height) {
    assert(p == &init_calls && mode == 0 && x == 2 && y == 3 && height == 100);
    if (!width) return false;
    assert(atomic_exchange(&keyboard_event, 738) == 0);
    return true;
}

void *SteamAPI_SteamUser_v023(void) { return &init_calls; }
static uint64_t ticket_call;
static uint8_t ticket_marker;
static bool ticket_pending;
static int ticket_polls;
uint64_t SteamAPI_ISteamUser_RequestEncryptedAppTicket(void *p, void *data, int n) {
    assert(!ticket_pending && n == 1);
    ticket_pending = true;
    ticket_marker = *(uint8_t *)data;
    ticket_polls = 0;
    return ++ticket_call;
}
bool SteamAPI_ISteamUtils_IsAPICallCompleted(void *p, uint64_t call, bool *failed) {
    assert(ticket_pending && call == ticket_call);
    *failed = false;
    return ticket_polls++ > 0;
}
bool SteamAPI_ISteamUtils_GetAPICallResult(void *p, uint64_t call, void *response, int size, int type, bool *failed) {
    assert(ticket_pending && call == ticket_call && type == 154 && size == sizeof(int32_t));
    *failed = false;
    *(int32_t *)response = ticket_marker == 0 ? 2 : 1;
    if (!ticket_marker) ticket_pending = false;
    return true;
}
bool SteamAPI_ISteamUser_GetEncryptedAppTicket(void *p, void *dest, int capacity, uint32_t *size) {
    assert(ticket_pending && ticket_marker);
    *size = 1025;
    if (capacity < 1025) return false;
    memset(dest, ticket_marker, *size);
    ticket_pending = false;
    return true;
}
int SteamAPI_ISteamUser_BeginAuthSession(void *p, const void *ticket, int len, uint64_t id) {
    assert(p == &init_calls && id == peer && len == 3);
    if (!memcmp(ticket, "bad", 3)) { ++rejected; return 1; }
    assert(!active && !memcmp(ticket, "\x01\x00\xff", 3));
    ++accepted;
    active = auth_thread = true;
    return 0;
}
void SteamAPI_ISteamUser_EndAuthSession(void *p, uint64_t id) {
    assert(p == &init_calls && active && id == peer);
    active = false;
    ++ended;
}
bool SteamAPI_ManualDispatch_GetNextCallback(int pipe, struct callback_msg *msg) {
    int key = atomic_load(&hold_keyboard) && !atomic_load(&release_keyboard) ? 0 : atomic_exchange(&keyboard_event, 0);
    if (key) {
        dispatching_keyboard = true;
        *msg = (struct callback_msg){1, key, &dismissed, key == 714 ? sizeof(dismissed) : 1};
        return true;
    }
    if (!auth_thread || !active || delivered == 6) return false;
    assert(freed == delivered);
    static struct overlay_activation overlay;
    static uint64_t join[2];
    static struct order_event order;
    if (delivered < 2) {
        overlay = (struct overlay_activation){delivered ? 0 : 2, !delivered, 480, 123};
        *msg = (struct callback_msg){1, 331, &overlay, sizeof(overlay)};
    } else if (delivered == 2) {
        join[0] = peer + 4; join[1] = peer;
        *msg = (struct callback_msg){1, 333, join, sizeof(join)};
    } else if (delivered == 3) {
        order = (struct order_event){480, UINT64_MAX - 9, 1};
        *msg = (struct callback_msg){1, 152, &order, sizeof(order)};
    } else {
        validation = (struct validation){peer, delivered == 5 ? 10 : 0, peer + 1};
        *msg = (struct callback_msg){1, 143, &validation, sizeof(validation)};
    }
    ++delivered;
    return true;
}
void SteamAPI_ManualDispatch_FreeLastCallback(int pipe) {
    if (dispatching_keyboard) { dispatching_keyboard = false; return; }
    assert(auth_thread && freed + 1 == delivered);
    memset(&validation, 0, sizeof(validation));
    ++freed;
}
void SteamAPI_Shutdown(void) { assert(false); }
void *SteamAPI_SteamUtils_v010(void) { return &init_calls; }
uint32_t SteamAPI_ISteamUtils_GetAppID(void *p) { return 480; }
void *SteamAPI_SteamInventory_v003(void) { return &init_calls; }
void *SteamAPI_SteamInput_v006(void) { return &init_calls; }
int SteamAPI_ISteamInput_GetConnectedControllers(void *p, uint64_t *handles) {
    assert(p == &init_calls); handles[0] = UINT64_MAX - 7; return 1;
}
#pragma pack(push, 1)
struct digital_action { bool pressed, active; };
struct analog_action { int mode; float x, y; bool active; };
#pragma pack(pop)
struct digital_action SteamAPI_ISteamInput_GetDigitalActionData(void *p, uint64_t controller, uint64_t action) {
    assert(p == &init_calls && controller == UINT64_MAX - 7 && action == 11);
    return (struct digital_action){false, true};
}
struct analog_action SteamAPI_ISteamInput_GetAnalogActionData(void *p, uint64_t controller, uint64_t action) {
    assert(p == &init_calls && controller == UINT64_MAX - 7 && action == 22);
    return (struct analog_action){6, -0.5f, 0.25f, true};
}
const char *SteamAPI_ISteamInput_GetStringForActionOrigin(void *p, int origin) {
    assert(p == &init_calls && (origin == 493 || origin == 494));
    if (origin == 493) atomic_store(&release_keyboard, true);
    return "SDK 1.64 origin";
}

static bool batch;
void *SteamAPI_SteamFriends_v018(void) { return &init_calls; }
static void (*warning_hook)(int, const char *);
void SteamAPI_ISteamUtils_SetWarningMessageHook(void *p, void (*hook)(int, const char *)) {
    assert(p == &init_calls);
    warning_hook = hook;
}
bool SteamAPI_ISteamFriends_HasFriend(void *p, uint64_t id, int flags) {
    assert(p == &init_calls && id == peer);
    if (warning_hook) warning_hook(1, "fixture warning");
    return flags == 4;
}
const char *SteamAPI_ISteamFriends_GetPlayerNickname(void *p, uint64_t id) {
    assert(p == &init_calls);
    return id == peer ? "Player Two" : "";
}
int SteamAPI_ISteamFriends_GetFriendsGroupCount(void *p) { return 2; }
int16_t SteamAPI_ISteamFriends_GetFriendsGroupIDByIndex(void *p, int i) { return i ? 8 : -2; }
const char *SteamAPI_ISteamFriends_GetFriendsGroupName(void *p, int16_t id) {
    return id == -2 ? "Party" : id == 8 ? "Empty" : NULL;
}
int SteamAPI_ISteamFriends_GetFriendsGroupMembersCount(void *p, int16_t id) { return id == -2 ? 2 : 0; }
void SteamAPI_ISteamFriends_GetFriendsGroupMembersList(void *p, int16_t id, uint64_t *ids, int count) {
    assert(id == -2 && count == 2); ids[0] = peer;
}
static uint32_t mask;
void *SteamAPI_SteamRemoteStorage_v016(void) { return &mask; }
bool SteamAPI_ISteamRemoteStorage_BeginFileWriteBatch(void *p) {
    if (batch) return false;
    return batch = true;
}
bool SteamAPI_ISteamRemoteStorage_EndFileWriteBatch(void *p) {
    if (!batch) return false;
    batch = false;
    return true;
}
bool SteamAPI_ISteamRemoteStorage_SetSyncPlatforms(void *p, const char *name, uint32_t value) {
    assert(p == &mask && batch && !strcmp(name, "cross.sav"));
    mask = value;
    return true;
}
uint32_t SteamAPI_ISteamRemoteStorage_GetSyncPlatforms(void *p, const char *name) {
    assert(p == &mask && !strcmp(name, "cross.sav"));
    return mask;
}
bool SteamAPI_ISteamRemoteStorage_GetQuota(void *p, uint64_t *total, uint64_t *available) {
    *total = UINT64_MAX; *available = UINT64_MAX - 1; return true;
}

int SteamAPI_ISteamUser_GetAvailableVoice(void *p, uint32_t *compressed, uint32_t *unused, uint32_t hz) {
    assert(!unused && !hz);
    *compressed = 0;
    return 3;
}
int SteamAPI_ISteamUser_DecompressVoice(void *p, const uint8_t *src, uint32_t len,
        uint8_t *dst, uint32_t capacity, uint32_t *written, uint32_t hz) {
    assert(len == 1 && src[0] == 123 && hz == 24000);
    *written = 24000;
    if (capacity < *written) return 4;
    memset(dst, 42, *written);
    return 0;
}

bool SteamAPI_ISteamInventory_GetAllItems(void *p, int *handle) {
    *handle = atomic_fetch_add(&submissions, 1);
    assert(*handle < 32);
    return true;
}
bool SteamAPI_ISteamInventory_GetItemsByID(void *p, int *handle, const uint64_t *ids, uint32_t n) {
    assert(n == 1);
    SteamAPI_ISteamInventory_GetAllItems(p, handle);
    modes[*handle] = *ids == item_id ? 0 : (int)*ids;
    return modes[*handle] != 3;
}
bool SteamAPI_ISteamInventory_SerializeResult(void *p, int h, void *buf, uint32_t *size) {
    assert(p == &init_calls && !destroyed[h] && kind[h]);
    if (modes[h] == 7) return false;
    char bytes[24];
    int len = snprintf(bytes, sizeof bytes, "receipt-%d-%d", kind[h], h);
    assert(len > 0 && (uint32_t)len < sizeof bytes);
    if (!buf) { *size = (uint32_t)len; return true; }
    assert(*size == (uint32_t)len);
    memcpy(buf, bytes, (size_t)len);
    return true;
}
bool SteamAPI_ISteamInventory_ConsumeItem(void *p, int *handle, uint64_t item, uint32_t quantity) {
    assert(p == &init_calls && quantity > 0 && item != UINT64_MAX);
    *handle = atomic_fetch_add(&submissions, 1);
    assert(*handle < 32);
    kind[*handle] = 1;
    modes[*handle] = item == item_id - 1 ? 7 : 0;
    return item == item_id || item == item_id - 1;
}
bool SteamAPI_ISteamInventory_ExchangeItems(void *p, int *handle,
        const int32_t *defs, const uint32_t *generate, uint32_t generate_len,
        const uint64_t *items, const uint32_t *destroy, uint32_t destroy_len) {
    assert(p == &init_calls && (generate_len || destroy_len));
    for (uint32_t i = 0; i < generate_len; ++i) assert(generate[i] > 0);
    for (uint32_t i = 0; i < destroy_len; ++i) assert(items[i] != UINT64_MAX && destroy[i] > 0);
    *handle = atomic_fetch_add(&submissions, 1);
    assert(*handle < 32);
    kind[*handle] = 2;
    modes[*handle] = generate_len == 2 ? 7 : 0;
    for (uint32_t i = 0; i < destroy_len; ++i)
        if (items[i] != item_id) return false;
    return true;
}
int SteamAPI_ISteamInventory_GetResultStatus(void *p, int h) {
    assert(!destroyed[h]);
    if (!polls[h]++ || modes[h] == 5) return 22;
    return modes[h] == 1 ? 20 : 1;
}
uint32_t SteamAPI_ISteamInventory_GetResultTimestamp(void *p, int h) { return 1234; }

#pragma pack(push, 4)
struct item { uint64_t id; int32_t definition; uint16_t quantity, flags; };
#pragma pack(pop)
bool SteamAPI_ISteamInventory_GetResultItems(void *p, int h, struct item *out, uint32_t *n) {
    if (modes[h] == 2) { *n = UINT32_MAX; return true; }
    if (out) { assert(*n == 1); *out = (struct item){item_id, 8, 2, 1}; }
    *n = 1;
    return true;
}
bool SteamAPI_ISteamInventory_GetResultItemProperty(void *p, int h, uint32_t i, const char *name, char *out, uint32_t *n) {
    assert(i == 0);
    if (name && modes[h] == 4) return false;
    const char *value = !name ? "quantity,name" : strcmp(name, "name") == 0 ? "Token" : "65538";
    if (out) { assert(*n == strlen(value) + 1); strcpy(out, value); }
    *n = (uint32_t)strlen(value) + 1;
    return true;
}
void SteamAPI_ISteamInventory_DestroyResult(void *p, int h) {
    assert(h >= 0 && h < atomic_load(&submissions) && !destroyed[h]);
    destroyed[h] = true;
    atomic_fetch_add(&drops, 1);
}
__attribute__((destructor)) static void verify_cleanup(void) {
    assert(atomic_load(&init_calls) == 2);
    assert(atomic_load(&submissions) == 16);
    assert(atomic_load(&drops) == atomic_load(&submissions));
    int consumes = 0, exchanges = 0;
    for (int i = 0; i < 32; ++i) {
        consumes += kind[i] == 1;
        exchanges += kind[i] == 2;
    }
    assert(consumes == 3 && exchanges == 4);
    assert(accepted == 1 && rejected == 1 && ended == 1 && freed == 6 && !active);
    fputs("auth fixture: rejected submission, accepted submission, OK then identity failure, session ended\n", stderr);
    fputs("inventory fixture: 1 failed init, 1 successful init, 2 refused and 2 unserializable mutations, 16 results destroyed\n", stderr);
}
