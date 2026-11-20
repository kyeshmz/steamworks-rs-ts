#include "../deviation-native/sdk.c"

void *SteamAPI_SteamRemotePlay_v004(void) { return &app_id; }
uint32_t SteamAPI_ISteamRemotePlay_GetSessionCount(void *self) {
    assert(self == &app_id);
    return fixture_case("too-many") ? 4097 : 3;
}
uint32_t SteamAPI_ISteamRemotePlay_GetSessionID(void *self, int32_t index) {
    assert(self == &app_id && index >= 0 && index < 3);
    const uint32_t ids[] = {UINT32_MAX, 0, 42};
    return ids[index];
}
uint64_t SteamAPI_ISteamRemotePlay_GetSessionSteamID(void *self, uint32_t id) {
    assert(self == &app_id && (id == UINT32_MAX || (id >= 7 && id <= 10)));
    return id == 7 ? 1 : UINT64_C(76561198012345678);
}
const char *SteamAPI_ISteamRemotePlay_GetSessionClientName(void *self, uint32_t id) {
    assert(self == &app_id);
    if (id == UINT32_MAX) return "Living Room";
    if (id == 7) return "Unknown Device";
    if (id >= 8 && id <= 10) return "Portable";
    return NULL;
}
int32_t SteamAPI_ISteamRemotePlay_GetSessionClientFormFactor(void *self, uint32_t id) {
    assert(self == &app_id && (id == UINT32_MAX || (id >= 7 && id <= 10)));
    return id == 7 ? 0 : id == UINT32_MAX ? 4 : (int32_t)id - 7;
}
bool SteamAPI_ISteamRemotePlay_BGetSessionClientResolution(void *self, uint32_t id, int32_t *width, int32_t *height) {
    assert(self == &app_id && (id == UINT32_MAX || (id >= 7 && id <= 10)));
    if (id == 7) return false;
    if (id == 10) { *width = -1; *height = 1; return true; }
    *width = 1920;
    *height = 1080;
    return true;
}
bool SteamAPI_ISteamRemotePlay_BSendRemotePlayTogetherInvite(void *self, uint64_t friend_id) {
    assert(self == &app_id);
    assert(friend_id == UINT64_C(76561198012345678) || friend_id == UINT64_MAX);
    return friend_id != UINT64_MAX;
}
