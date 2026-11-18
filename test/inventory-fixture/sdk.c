#include <assert.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdatomic.h>
#include <string.h>

static atomic_int init_calls, submissions, drops;
static int modes[32], polls[32];
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
bool SteamAPI_ManualDispatch_GetNextCallback(int pipe, void *msg) { return false; }
void SteamAPI_Shutdown(void) { assert(false); }
void *SteamAPI_SteamUtils_v010(void) { return &init_calls; }
void *SteamAPI_SteamUserStats_v012(void) { return &init_calls; }
bool SteamAPI_ISteamUserStats_RequestCurrentStats(void *p) { return true; }
uint32_t SteamAPI_ISteamUtils_GetAppID(void *p) { return 480; }
void *SteamAPI_SteamInventory_v003(void) { return &init_calls; }

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
    assert(atomic_load(&submissions) == 9);
    assert(atomic_load(&drops) == atomic_load(&submissions));
    fputs("inventory fixture: 1 failed init, 1 successful init, 9 results destroyed\n", stderr);
}
