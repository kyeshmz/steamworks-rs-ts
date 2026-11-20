#include <assert.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>

typedef struct Message {
    void *data;
    int32_t size;
    uint32_t connection;
    unsigned char identity[136];
    int64_t connection_user_data;
    int64_t received;
    int64_t number;
    void *free_data;
    void *release;
    int32_t channel;
    int32_t flags;
    int64_t user_data;
    uint16_t lane;
    uint16_t padding;
} Message;

static int sockets, utils, networking, messages, allocated;

int SteamAPI_InitFlat(char *error) { (void)error; return 0; }
void SteamAPI_ManualDispatch_Init(void) {}
int SteamAPI_GetHSteamPipe(void) { return 42; }
void SteamAPI_Shutdown(void) {}
uint64_t SteamAPI_SteamNetworkingIdentity_GetSteamID64(void *identity) {
    uint64_t id;
    memcpy(&id, (uint8_t *)identity + 8, sizeof(id));
    return id;
}
void SteamAPI_SteamNetworkingIdentity_SetSteamID64(void *identity, uint64_t id) {
    int32_t type = 16, size = 8;
    memcpy(identity, &type, sizeof(type));
    memcpy((uint8_t *)identity + 4, &size, sizeof(size));
    memcpy((uint8_t *)identity + 8, &id, sizeof(id));
}

void *SteamAPI_SteamNetworking_v006(void) { return &networking; }
bool SteamAPI_ISteamNetworking_AcceptP2PSessionWithUser(void *self, uint64_t peer) {
    assert(self == &networking);
    assert(peer == UINT64_C(76561198012345678) || peer == UINT64_C(76561198012345679));
    return peer == UINT64_C(76561198012345678);
}
bool SteamAPI_ISteamNetworking_CloseP2PSessionWithUser(void *self, uint64_t peer) {
    assert(self == &networking);
    assert(peer == UINT64_C(76561198012345678) || peer == UINT64_C(76561198012345679));
    return peer == UINT64_C(76561198012345679);
}
bool SteamAPI_ISteamNetworking_SendP2PPacket(void *self, uint64_t peer, const void *data, uint32_t size, int mode, int channel) {
    assert(self == &networking && peer == UINT64_C(76561198012345679));
    assert(size == 3 && memcmp(data, "\x00\xff\x41", 3) == 0 && mode == 2);
    assert(channel == 0 || channel == 2147483647);
    return true;
}
bool SteamAPI_ISteamNetworking_IsP2PPacketAvailable(void *self, uint32_t *size, int channel) {
    assert(self == &networking && (channel == 0 || channel == 2147483647));
    *size = 3;
    return true;
}
bool SteamAPI_ISteamNetworking_ReadP2PPacket(void *self, void *data, uint32_t capacity, uint32_t *read, uint64_t *peer, int channel) {
    assert(self == &networking && capacity == 5 && (channel == 0 || channel == 2147483647));
    memcpy(data, "\x00\xff\x41", 3);
    *read = 3;
    *peer = UINT64_C(76561198012345679);
    return true;
}
typedef struct {
    uint8_t active, connecting, error, relay;
    int32_t bytes, packets;
    uint32_t ip;
    uint16_t port;
} P2PState;
bool SteamAPI_ISteamNetworking_GetP2PSessionState(void *self, uint64_t peer, P2PState *state) {
    assert(self == &networking && peer == UINT64_C(76561198012345678));
    *state = (P2PState){1, 0, 4, 1, 120, 2, 0xc0000201, 27015};
    return true;
}

void *SteamAPI_SteamNetworkingMessages_SteamAPI_v002(void) { return &messages; }
int SteamAPI_ISteamNetworkingMessages_GetSessionConnectionInfo(void *self, void *identity, uint8_t *info, uint8_t *status) {
    assert(self == &messages && SteamAPI_SteamNetworkingIdentity_GetSteamID64(identity) == UINT64_C(76561198012345678));
    int32_t listen = 99, end = 42, ping = 27;
    float local = 0.5f, remote = 0.75f;
    memcpy(info, identity, 136);
    memcpy(info + 144, &listen, 4);
    memcpy(info + 180, &end, 4);
    memcpy(status + 4, &ping, 4);
    memcpy(status + 8, &local, 4);
    memcpy(status + 12, &remote, 4);
    return 3;
}

#pragma pack(push, 4)
typedef struct { int32_t user, id; void *data; int32_t size; } Callback;
#pragma pack(pop)
static uint8_t request[136], failure[696], connection[704];
static int event_index;
void SteamAPI_ManualDispatch_RunFrame(int pipe) { assert(pipe == 42); event_index = 0; }
bool SteamAPI_ManualDispatch_GetNextCallback(int pipe, Callback *event) {
    assert(pipe == 42);
    if (event_index == 0) *event = (Callback){1, 1251, request, sizeof(request)};
    else if (event_index == 1) *event = (Callback){1, 1252, failure, sizeof(failure)};
    else if (event_index == 2) *event = (Callback){1, 1221, connection, sizeof(connection)};
    else return false;
    event_index++;
    return true;
}
void SteamAPI_ManualDispatch_FreeLastCallback(int pipe) { assert(pipe == 42); }

void *SteamAPI_SteamNetworkingSockets_SteamAPI_v012(void) { return &sockets; }
void *SteamAPI_SteamNetworkingUtils_SteamAPI_v004(void) { return &utils; }

int64_t SteamAPI_ISteamNetworkingSockets_GetConnectionUserData(void *self, uint32_t connection) {
    assert(self == &sockets);
    return connection == 123 ? INT64_C(9007199254740993) : -1;
}
bool SteamAPI_ISteamNetworkingSockets_SetConnectionUserData(void *self, uint32_t connection, int64_t value) {
    assert(self == &sockets && connection == 123 && value == -INT64_C(9007199254740993));
    return true;
}
bool SteamAPI_ISteamNetworkingSockets_GetConnectionName(void *self, uint32_t connection, char *name, int32_t capacity) {
    assert(self == &sockets && capacity >= 8);
    if (connection != 123) return false;
    memcpy(name, "fixture", 8);
    return true;
}
void SteamAPI_ISteamNetworkingSockets_SetConnectionName(void *self, uint32_t connection, const char *name) {
    assert(self == &sockets && connection == 123 && strcmp(name, "test") == 0);
}
int SteamAPI_ISteamNetworkingSockets_FlushMessagesOnConnection(void *self, uint32_t connection) {
    assert(self == &sockets && connection == 123);
    return 1;
}
int SteamAPI_ISteamNetworkingSockets_InitAuthentication(void *self) {
    assert(self == &sockets);
    return 100;
}
int SteamAPI_ISteamNetworkingSockets_GetAuthenticationStatus(void *self, uint8_t *details) {
    assert(self == &sockets);
    int32_t status = 100;
    memcpy(details, &status, 4);
    memcpy(details + 4, "ready", 6);
    return 100;
}
int SteamAPI_ISteamNetworkingSockets_GetConnectionRealTimeStatus(void *self, uint32_t connection, uint8_t *status, int32_t count, uint8_t *lanes) {
    assert(self == &sockets && connection == 123 && count == 2 && lanes);
    int32_t state = 3, ping = 21, pending = 64;
    int64_t queue = INT64_C(9007199254740993);
    memcpy(status, &state, 4);
    memcpy(status + 4, &ping, 4);
    memcpy(lanes, &pending, 4);
    memcpy(lanes + 16, &queue, 8);
    return 1;
}
int SteamAPI_ISteamNetworkingSockets_ConfigureConnectionLanes(void *self, uint32_t connection, int32_t count, const int32_t *priorities, const uint16_t *weights) {
    assert(self == &sockets && connection == 123 && count == 2);
    assert(priorities[0] == -1 && priorities[1] == 42 && weights[0] == 1 && weights[1] == 65535);
    return 1;
}

Message *SteamAPI_ISteamNetworkingUtils_AllocateMessage(void *self, int32_t size) {
    assert(self == &utils && size >= 0);
    if (size == 17) return NULL;
    Message *message = calloc(1, sizeof(*message));
    assert(message);
    if (size) {
        message->data = malloc(size);
        assert(message->data);
    }
    message->size = size;
    allocated++;
    return message;
}

void SteamAPI_SteamNetworkingMessage_t_Release(Message *message) {
    assert(message && allocated > 0);
    free(message->data);
    free(message);
    allocated--;
}

void SteamAPI_ISteamNetworkingSockets_SendMessages(void *self, int count, Message **messages, int64_t *results) {
    assert(self == &sockets && count == 2 && allocated == 2);
    assert(messages[0]->connection == 123 && messages[0]->lane == 7 && messages[0]->flags == 8);
    assert(messages[0]->size == 3 && memcmp(messages[0]->data, "\x00\xff\x41", 3) == 0);
    assert(messages[1]->connection == 456 && messages[1]->lane == 0 && messages[1]->flags == 0);
    assert(messages[1]->size == 0);
    results[0] = INT64_C(9007199254740993);
    results[1] = -8;
    for (int i = 0; i < count; ++i) SteamAPI_SteamNetworkingMessage_t_Release(messages[i]);
    assert(allocated == 0);
}
