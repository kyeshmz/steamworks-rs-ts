#include <assert.h>
#include <stdbool.h>
#include <stdint.h>
#include <string.h>

static bool live, logged, pending;
static int config, frame, connection_error[2] = {3, 0};

int SteamInternal_GameServer_Init_V2(uint32_t ip, uint16_t game, uint16_t query, int mode,
                                     const char *version, const char *interfaces, char *message) {
    assert(!live && ip == 0x7f000001 && game == 12345 && query == 12346 && mode == 2);
    if (!strcmp(version, "fail")) { strcpy(message, "fixture init refused"); return 1; }
    assert(!strcmp(version, "1.0") && interfaces);
    live = true;
    config = 0;
    frame = 0;
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
int SteamGameServer_GetHSteamPipe(void) { assert(live); return 17; }
void SteamAPI_ManualDispatch_RunFrame(int pipe) { assert(pipe == 17 && logged); ++frame; pending = frame <= 3; }
#pragma pack(push, 4)
struct callback_msg { int user, id; void *data; int size; };
#pragma pack(pop)
bool SteamAPI_ManualDispatch_GetNextCallback(int pipe, struct callback_msg *msg) {
    if (!pending) return false;
    if (frame == 1) *msg = (struct callback_msg){1, 101, &live, 1};
    else *msg = (struct callback_msg){1, frame == 2 ? 103 : 102, connection_error, sizeof(connection_error)};
    pending = false;
    return true;
}
void SteamAPI_ManualDispatch_FreeLastCallback(int pipe) { assert(pipe == 17); }
uint64_t SteamAPI_ISteamGameServer_GetSteamID(void *p) { assert(logged); return UINT64_MAX - 1; }
void SteamGameServer_Shutdown(void) { assert(live && logged && !pending); live = logged = false; }
