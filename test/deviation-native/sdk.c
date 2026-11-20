#define _POSIX_C_SOURCE 200809L
#include <assert.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <pthread.h>
#include <time.h>

static const uint64_t steam_id = UINT64_C(76561198012345678);
static uint32_t app_id = 3104020;
static int32_t stat;
static bool achievement;
static uint32_t next_handle;
static unsigned int outstanding, canceled, stores, canceled_mask, name_reads, id_reads;
static pthread_mutex_t lock = PTHREAD_MUTEX_INITIALIZER;

#pragma pack(push, 4)
typedef struct { int32_t user, id; void *payload; int32_t size; } Callback;
#pragma pack(pop)
typedef struct { uint32_t handle; int32_t result, length; uint8_t bytes[2560]; } Ticket;
static struct { int32_t id, size; union { Ticket ticket; uint8_t stats[20]; } data; } queue[16];
static unsigned int first, last, current, delivered_mask;
static struct timespec last_issued;

static bool fixture_case(const char *name) {
    const char *value = getenv("FIXTURE_CASE");
    return value && strcmp(value, name) == 0;
}
static bool overlap_case(void) {
    const char *value = getenv("FIXTURE_CASE");
    return value && strncmp(value, "overlap-", 8) == 0;
}
static bool paired_case(void) {
    return fixture_case("overlap-session") || fixture_case("overlap-web")
        || fixture_case("overlap-late-session") || fixture_case("overlap-late-web");
}

bool SteamAPI_RestartAppIfNecessary(uint32_t id) {
    assert(id == app_id);
    const char *restart = getenv("FIXTURE_RESTART");
    assert(restart);
    return strcmp(restart, "1") == 0;
}
int32_t SteamAPI_InitFlat(char *error) {
    const char *id = getenv("SteamAppId");
    assert(id && strcmp(id, "3104020") == 0);
    return 0;
}
void SteamAPI_Shutdown(void) {}
void *SteamAPI_SteamUser_v023(void) { return &app_id; }
void *SteamAPI_SteamUtils_v010(void) { return &app_id; }
void *SteamAPI_SteamFriends_v017(void) { return &app_id; }
void *SteamAPI_SteamFriends_v018(void) { return &app_id; }
void *SteamAPI_SteamUserStats_v012(void) { return &app_id; }
void *SteamAPI_SteamUserStats_v013(void) { return &app_id; }
void *SteamAPI_SteamApps_v008(void) { return &app_id; }
void *SteamAPI_SteamApps_v009(void) { return &app_id; }
uint32_t SteamAPI_ISteamUtils_GetAppID(void *self) { assert(self == &app_id); return app_id; }
uint64_t SteamAPI_ISteamUser_GetSteamID(void *self) { assert(self == &app_id); id_reads++; return steam_id; }
const char *SteamAPI_ISteamFriends_GetPersonaName(void *self) { assert(self == &app_id); name_reads++; return "Fixture Steam player"; }
bool SteamAPI_ISteamApps_BIsAppInstalled(void *self, uint32_t id) { assert(self == &app_id); return id == app_id; }

bool SteamAPI_ISteamUserStats_RequestCurrentStats(void *self) {
    assert(self == &app_id);
    return true;
}
bool SteamAPI_ISteamUserStats_GetStatInt32(void *self, const char *name, int32_t *value) {
    assert(self == &app_id && strcmp(name, "GAMES_WON") == 0); *value = stat; return true;
}
bool SteamAPI_ISteamUserStats_SetStatInt32(void *self, const char *name, int32_t value) {
    assert(self == &app_id && strcmp(name, "GAMES_WON") == 0); stat = value; return true;
}
bool SteamAPI_ISteamUserStats_StoreStats(void *self) { assert(self == &app_id); stores++; return true; }
bool SteamAPI_ISteamUserStats_ResetAllStats(void *self, bool achievements_too) {
    assert(self == &app_id); stat = 0; if (achievements_too) achievement = false; return true;
}
bool SteamAPI_ISteamUserStats_SetAchievement(void *self, const char *name) {
    assert(self == &app_id && strcmp(name, "FIXTURE_ACHIEVEMENT") == 0); achievement = true; return true;
}
bool SteamAPI_ISteamUserStats_ClearAchievement(void *self, const char *name) {
    assert(self == &app_id && strcmp(name, "FIXTURE_ACHIEVEMENT") == 0); achievement = false; return true;
}
bool SteamAPI_ISteamUserStats_GetAchievement(void *self, const char *name, bool *value) {
    assert(self == &app_id && strcmp(name, "FIXTURE_ACHIEVEMENT") == 0); *value = achievement; return true;
}

void SteamAPI_SteamNetworkingIdentity_Clear(uint8_t *identity) { memset(identity, 0, 136); }
void SteamAPI_SteamNetworkingIdentity_SetSteamID64(uint8_t *identity, uint64_t id) {
    int32_t type = 16, size = 8;
    memcpy(identity, &type, 4); memcpy(identity + 4, &size, 4); memcpy(identity + 8, &id, 8);
}
static uint32_t make_ticket(int32_t callback_id, uint8_t marker) {
    pthread_mutex_lock(&lock);
    assert(last < 16);
    uint32_t handle = ++next_handle;
    outstanding++;
    unsigned int i = last++;
    queue[i].id = callback_id;
    queue[i].size = callback_id == 168 ? sizeof(Ticket) : 8;
    bool fail = fixture_case("overlap-failure") || (fixture_case("overlap-session") && marker == 2);
    queue[i].data.ticket = (Ticket){.handle = handle, .result = fail ? 2 : 1, .length = 4, .bytes = {0xca, 0xfe, 0x00, marker ? marker : (uint8_t)handle}};
    if (fixture_case("overlap-length") && marker >= 1 && marker <= 3) {
        const int32_t lengths[] = {-1, 0, 2561};
        queue[i].data.ticket.length = lengths[marker - 1];
    }
    assert(clock_gettime(CLOCK_MONOTONIC, &last_issued) == 0);
    pthread_mutex_unlock(&lock);
    return handle;
}
uint32_t SteamAPI_ISteamUser_GetAuthSessionTicket(void *self, uint8_t *buffer, int32_t capacity, uint32_t *size, const uint8_t *identity) {
    int32_t type, identity_size;
    uint64_t id;
    assert(self == &app_id && identity && capacity >= 3);
    memcpy(&type, identity, 4); memcpy(&identity_size, identity + 4, 4); memcpy(&id, identity + 8, 8);
    assert(type == 16 && identity_size == 8 && identity[135] == 0);
    assert(overlap_case() ? id >= steam_id + 1 && id <= steam_id + 5 : id == steam_id);
    uint8_t marker = overlap_case() ? (uint8_t)(id - steam_id) : 0;
    uint32_t handle = make_ticket(163, marker);
    buffer[0] = 0xf1; buffer[1] = 0; buffer[2] = marker ? marker : (uint8_t)handle; *size = 3;
    return handle;
}
uint32_t SteamAPI_ISteamUser_GetAuthTicketForWebApi(void *self, const char *identity) {
    assert(self == &app_id && identity);
    uint8_t marker = 0;
    if (overlap_case()) {
        assert(strncmp(identity, "peer-", 5) == 0 && identity[5] >= '1' && identity[5] <= '5' && identity[6] == '\0');
        marker = (uint8_t)(identity[5] - '0');
    } else {
        assert(strcmp(identity, "deviation-fixture") == 0);
    }
    return make_ticket(168, marker);
}
void SteamAPI_ISteamUser_CancelAuthTicket(void *self, uint32_t handle) {
    pthread_mutex_lock(&lock);
    assert(self == &app_id && handle > 0 && handle <= next_handle && handle < 32 && outstanding > 0 && !(canceled_mask & (1U << handle)));
    canceled_mask |= 1U << handle;
    outstanding--; canceled++;
    fprintf(stderr, "fixture canceled ticket %u\n", handle);
    pthread_mutex_unlock(&lock);
}
int32_t SteamAPI_GetHSteamPipe(void) { return 42; }
void SteamAPI_ManualDispatch_Init(void) {}
void SteamAPI_ManualDispatch_RunFrame(int32_t pipe) { assert(pipe == 42); }
bool SteamAPI_ManualDispatch_GetNextCallback(int32_t pipe, Callback *callback) {
    assert(pipe == 42);
    pthread_mutex_lock(&lock);
    if (first == last) { pthread_mutex_unlock(&lock); return false; }
    if (overlap_case()) {
        struct timespec now;
        assert(clock_gettime(CLOCK_MONOTONIC, &now) == 0);
        long long elapsed = (now.tv_sec - last_issued.tv_sec) * 1000000000LL + now.tv_nsec - last_issued.tv_nsec;
        if ((paired_case() && last < 2) || elapsed < 200000000LL) {
            pthread_mutex_unlock(&lock);
            return false;
        }
    }
    unsigned int i = first;
    if ((fixture_case("overlap-session") || fixture_case("overlap-web")) && first < 2) i = 1 - first;
    while (delivered_mask & (1U << i)) i++;
    assert(i < last);
    current = i;
    *callback = (Callback){1, queue[i].id, &queue[i].data, queue[i].size};
    if (overlap_case()) fprintf(stderr, "fixture delivers callback handle %u\n", queue[i].data.ticket.handle);
    pthread_mutex_unlock(&lock);
    return true;
}
void SteamAPI_ManualDispatch_FreeLastCallback(int32_t pipe) {
    assert(pipe == 42);
    pthread_mutex_lock(&lock);
    assert(first < last);
    delivered_mask |= 1U << current;
    memset(&queue[current], 0, sizeof(queue[0]));
    first++;
    pthread_mutex_unlock(&lock);
}
__attribute__((destructor)) static void verify_cleanup(void) {
    if (fixture_case("deviation-game")) {
        assert(next_handle == 2 && outstanding == 2 && canceled == 0 && stores >= 1 && name_reads >= 1 && id_reads >= 1);
        fprintf(stderr, "fixture observed Fixture Steam player and SteamID64 76561198012345678 reads, 2 issued, 2 outstanding, 0 cancellations and %u stores\n", stores);
    } else if (next_handle) {
        assert(outstanding == 0 && canceled == next_handle && stores >= 1);
        fprintf(stderr, "fixture observed %u cancellations and %u stores\n", canceled, stores);
    }
}
