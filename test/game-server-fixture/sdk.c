#include <assert.h>
#include <stdbool.h>
#include <stdint.h>
#include <string.h>

static bool live, logged, pending;
static int config, stage, connection_error[2] = {3, 0};

#define AUTH_PEER ((uint64_t)1 << 63 | 1)
#define AUTH_OWNER UINT64_MAX

#pragma pack(push, 4)
struct validate_auth { uint64_t steam; uint32_t response; uint64_t owner; };
#pragma pack(pop)
static struct validate_auth auth_event[2];
static int auth_count, auth_next;

int SteamInternal_GameServer_Init_V2(uint32_t ip, uint16_t game, uint16_t query, int mode,
                                     const char *version, const char *interfaces, char *message) {
    assert(!live && ip == 0x7f000001 && game == 12345 && query == 12346 && mode == 2);
    if (!strcmp(version, "fail")) { strcpy(message, "fixture init refused"); return 1; }
    assert(!strcmp(version, "1.0") && interfaces);
    live = true;
    config = 0;
    stage = 0;
    auth_count = auth_next = 0;
    return 0;
}
void SteamAPI_ManualDispatch_Init(void) { assert(live); }
void *SteamAPI_SteamGameServer_v015(void) { assert(live); return &live; }
void SteamAPI_ISteamGameServer_SetDedicatedServer(void *p, bool value) { assert(p == &live && live && value); config |= 1; }
void SteamAPI_ISteamGameServer_SetProduct(void *p, const char *s) { assert(!strcmp(s, "480")); config |= 2; }
void SteamAPI_ISteamGameServer_SetGameDescription(void *p, const char *s) { assert(!strcmp(s, "Fixture")); config |= 4; }
void SteamAPI_ISteamGameServer_SetServerName(void *p, const char *s) { assert(!strcmp(s, "Room")); config |= 8; }
void SteamAPI_ISteamGameServer_SetMapName(void *p, const char *s) { assert(!strcmp(s, logged ? "next" : "map")); config |= 16; }
void SteamAPI_ISteamGameServer_SetMaxPlayerCount(void *p, int n) { assert(n == 8); config |= 32; }
void SteamAPI_ISteamGameServer_SetAdvertiseServerActive(void *p, bool value) { assert(value == logged); config |= 64; }
void SteamAPI_ISteamGameServer_LogOnAnonymous(void *p) { assert(live && !logged && config == 127); pending = logged = true; }
void SteamAPI_ISteamGameServer_LogOn(void *p, const char *s) { assert(!strcmp(s, "fixture-only")); SteamAPI_ISteamGameServer_LogOnAnonymous(p); }
int SteamAPI_ISteamGameServer_BeginAuthSession(void *p, const void *ticket, int len, uint64_t id) {
    assert(p == &live && live && logged && len > 0 && id == AUTH_PEER);
    if (len == 6 && !memcmp(ticket, "reject", 6)) return 1;
    assert(auth_next == auth_count);
    auth_event[0] = (struct validate_auth){AUTH_PEER, 0, AUTH_OWNER};
    auth_count = 1;
    auth_next = 0;
    if (len == 6 && !memcmp(ticket, "revoke", 6))
        auth_event[auth_count++] = (struct validate_auth){AUTH_PEER, 6, AUTH_OWNER};
    return 0;
}
void SteamAPI_ISteamGameServer_EndAuthSession(void *p, uint64_t id) {
    assert(p == &live && live && logged && id == AUTH_PEER && auth_next == auth_count);
}
int SteamGameServer_GetHSteamPipe(void) { assert(live); return 17; }
void SteamAPI_ManualDispatch_RunFrame(int pipe) { assert(pipe == 17 && logged); pending = stage < 3; }
#pragma pack(push, 4)
struct callback_msg { int user, id; void *data; int size; };
#pragma pack(pop)
bool SteamAPI_ManualDispatch_GetNextCallback(int pipe, struct callback_msg *msg) {
    if (pending) {
        if (stage == 0) *msg = (struct callback_msg){1, 101, &live, 1};
        else if (stage == 1) *msg = (struct callback_msg){1, 103, connection_error, sizeof(connection_error)};
        else *msg = (struct callback_msg){1, 102, connection_error, sizeof(connection_error)};
        ++stage;
        pending = false;
        return true;
    }
    if (auth_next < auth_count) {
        *msg = (struct callback_msg){1, 143, &auth_event[auth_next], sizeof(auth_event[0])};
        ++auth_next;
        return true;
    }
    return false;
}
void SteamAPI_ManualDispatch_FreeLastCallback(int pipe) { assert(pipe == 17); }
uint64_t SteamAPI_ISteamGameServer_GetSteamID(void *p) { assert(logged); return UINT64_MAX - 1; }
void SteamGameServer_Shutdown(void) { assert(live && logged && !pending); live = logged = false; }
