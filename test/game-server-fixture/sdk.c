#include <assert.h>
#include <stdbool.h>
#include <stdint.h>
#include <stddef.h>
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
static int server_event_next, packet_pending;
static uint16_t query_port;
static bool socket_open, child_open, message_pending, socket_message_pending;
static int socket_callback_pending, message_releases;
static bool message_request_pending, p2p_request_pending;
static uint64_t p2p_request = AUTH_PEER;
static unsigned char socket_event[704];

struct identity { int type, size; unsigned char data[128]; };
struct message {
    void *data;
    int size;
    uint32_t connection;
    struct identity peer;
    int64_t user_data, received, number;
    void *free_data, *release;
    int channel, flags;
    int64_t message_data;
    uint16_t lane, pad;
};
_Static_assert(offsetof(struct message, peer) == 16, "message identity offset");
_Static_assert(sizeof(struct message) == 216, "message size");
static struct message incoming;
static struct identity message_request;
static char incoming_data[] = "server-message";

void SteamAPI_SteamNetworkingIdentity_SetSteamID64(struct identity *id, uint64_t steam) {
    memset(id, 0, sizeof(*id));
    id->type = 16;
    id->size = 8;
    memcpy(id->data, &steam, sizeof(steam));
}
uint64_t SteamAPI_SteamNetworkingIdentity_GetSteamID64(struct identity *id) {
    uint64_t result;
    if (id->type != 16) return 0;
    memcpy(&result, id->data, sizeof(result));
    return result;
}
void *SteamAPI_SteamGameServerUtils_v010(void) { assert(live); return &config; }
uint32_t SteamAPI_ISteamUtils_GetAppID(void *p) { assert(p == &config); return 480; }
uint32_t SteamAPI_ISteamUtils_GetServerRealTime(void *p) { assert(p == &config); return 1700000000; }
void *SteamAPI_SteamGameServerNetworking_v006(void) { assert(live); return &stage; }
bool SteamAPI_ISteamNetworking_AcceptP2PSessionWithUser(void *p, uint64_t id) { assert(p == &stage && id == AUTH_PEER); return true; }
bool SteamAPI_ISteamNetworking_CloseP2PSessionWithUser(void *p, uint64_t id) { assert(p == &stage && id == AUTH_PEER); return true; }
struct p2p_state { uint8_t active, connecting, error, relay; int bytes, packets; uint32_t ip; uint16_t port; };
bool SteamAPI_ISteamNetworking_GetP2PSessionState(void *p, uint64_t id, struct p2p_state *state) {
    assert(p == &stage && id == AUTH_PEER);
    p2p_request_pending = true;
    *state = (struct p2p_state){1, 0, 4, 1, 12, 2, 0x01020304, 27015};
    return true;
}
void *SteamAPI_SteamGameServerNetworkingMessages_SteamAPI_v002(void) { assert(live); return &message_pending; }
int SteamAPI_ISteamNetworkingMessages_SendMessageToUser(void *p, struct identity *id, const void *data, uint32_t len, int flags, int channel) {
    assert(p == &message_pending && SteamAPI_SteamNetworkingIdentity_GetSteamID64(id) == AUTH_PEER);
    assert(len == 4 && !memcmp(data, "ping", 4) && flags == 8 && channel == 7);
    message_pending = true;
    SteamAPI_SteamNetworkingIdentity_SetSteamID64(&message_request, AUTH_PEER);
    message_request_pending = true;
    return 1;
}
bool SteamAPI_ISteamNetworkingMessages_AcceptSessionWithUser(void *p, struct identity *id) {
    assert(p == &message_pending && SteamAPI_SteamNetworkingIdentity_GetSteamID64(id) == AUTH_PEER);
    return true;
}
bool SteamAPI_ISteamNetworkingMessages_CloseSessionWithUser(void *p, struct identity *id) {
    assert(p == &message_pending && SteamAPI_SteamNetworkingIdentity_GetSteamID64(id) == AUTH_PEER);
    if (socket_open && child_open) socket_callback_pending = 3;
    return true;
}
int SteamAPI_ISteamNetworkingMessages_ReceiveMessagesOnChannel(void *p, int channel, struct message **out, int max) {
    assert(p == &message_pending && max == 2);
    if (channel == 8) return -1;
    if (channel == 9) {
        memset(&incoming, 0, sizeof(incoming));
        incoming.data = incoming_data;
        incoming.size = 512 * 1024 + 1;
        *out = &incoming;
        return 1;
    }
    assert(channel == 7);
    if (!message_pending) return 0;
    message_pending = false;
    memset(&incoming, 0, sizeof(incoming));
    incoming.data = incoming_data;
    incoming.size = sizeof(incoming_data) - 1;
    SteamAPI_SteamNetworkingIdentity_SetSteamID64(&incoming.peer, AUTH_OWNER);
    *out = &incoming;
    return 1;
}
void SteamAPI_SteamNetworkingMessage_t_Release(struct message *message) { assert(message == &incoming); ++message_releases; }
void *SteamAPI_SteamGameServerNetworkingSockets_SteamAPI_v012(void) { assert(live); return &socket_open; }
uint32_t SteamAPI_ISteamNetworkingSockets_CreateHostedDedicatedServerListenSocket(void *p, int port, int count, void *options) {
    assert(p == &socket_open && !socket_open && port == 0 && count == 0 && !options);
    socket_open = child_open = true;
    socket_callback_pending = 1;
    return 314;
}
int SteamAPI_ISteamNetworkingSockets_AcceptConnection(void *p, uint32_t connection) {
    assert(p == &socket_open && socket_open && child_open && connection == 700);
    socket_callback_pending = 2;
    return 1;
}
bool SteamAPI_ISteamNetworkingSockets_CloseConnection(void *p, uint32_t connection, int reason, const char *debug, bool linger) {
    assert(p == &socket_open && socket_open && child_open && connection == 700 && !reason && !debug && !linger);
    child_open = false;
    return true;
}
int SteamAPI_ISteamNetworkingSockets_SendMessageToConnection(void *p, uint32_t connection, const void *data, uint32_t len, int flags, int64_t *number) {
    assert(p == &socket_open && socket_open && child_open && connection == 700);
    assert(len == 4 && !memcmp(data, "play", 4) && flags == 8);
    socket_message_pending = true;
    *number = INT64_C(9007199254740993);
    return 1;
}
int SteamAPI_ISteamNetworkingSockets_ReceiveMessagesOnConnection(void *p, uint32_t connection, struct message **out, int max) {
    assert(p == &socket_open && socket_open && child_open && connection == 700 && max == 2);
    if (!socket_message_pending) return 0;
    socket_message_pending = false;
    memset(&incoming, 0, sizeof(incoming));
    incoming.data = incoming_data;
    incoming.size = sizeof(incoming_data) - 1;
    SteamAPI_SteamNetworkingIdentity_SetSteamID64(&incoming.peer, AUTH_PEER);
    *out = &incoming;
    return 1;
}
bool SteamAPI_ISteamNetworkingSockets_CloseListenSocket(void *p, uint32_t socket) {
    assert(p == &socket_open && socket_open && !child_open && socket == 314);
    socket_open = false;
    socket_callback_pending = 0;
    return true;
}

#pragma pack(push, 4)
struct approve_event { uint64_t user, owner; };
struct deny_event { uint64_t user; int reason; char text[128]; };
struct kick_event { uint64_t user; int reason; };
struct group_event { uint64_t user, group; bool member, officer; };
#pragma pack(pop)
static struct approve_event approve_event = { AUTH_PEER, AUTH_OWNER };
static struct deny_event deny_event = { AUTH_PEER, 5, "denied" };
static struct kick_event kick_event = { AUTH_PEER, 3 };
static struct group_event group_event = { AUTH_PEER, AUTH_OWNER, true, false };

int SteamInternal_GameServer_Init_V2(uint32_t ip, uint16_t game, uint16_t query, int mode,
                                     const char *version, const char *interfaces, char *message) {
    assert(!live && ip == 0x7f000001 && game == 12345 && (query == 12346 || query == 65535) && mode == 2);
    if (!strcmp(version, "fail")) { strcpy(message, "fixture init refused"); return 1; }
    assert(!strcmp(version, "1.0") && interfaces);
    live = true;
    config = 0;
    stage = 0;
    auth_count = auth_next = 0;
    server_event_next = packet_pending = 0;
    socket_open = child_open = message_pending = socket_message_pending = false;
    socket_callback_pending = message_releases = 0;
    message_request_pending = p2p_request_pending = false;
    query_port = query;
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
void SteamAPI_ISteamGameServer_SetBotPlayerCount(void *p, int n) { assert(p == &live && n == 2); }
void SteamAPI_ISteamGameServer_SetPasswordProtected(void *p, bool value) { assert(p == &live && value); }
void SteamAPI_ISteamGameServer_SetGameData(void *p, const char *s) { assert(p == &live && !strcmp(s, "mode=arena")); }
void SteamAPI_ISteamGameServer_SetModDir(void *p, const char *s) { assert(p == &live && !strcmp(s, "mod")); }
void SteamAPI_ISteamGameServer_SetGameTags(void *p, const char *s) { assert(p == &live && !strcmp(s, "ranked")); }
void SteamAPI_ISteamGameServer_SetKeyValue(void *p, const char *k, const char *v) { assert(p == &live && !strcmp(k, "mode") && !strcmp(v, "arena")); }
void SteamAPI_ISteamGameServer_ClearAllKeyValues(void *p) { assert(p == &live); }
bool SteamAPI_ISteamGameServer_HandleIncomingPacket(void *p, const void *data, int len, uint32_t ip, uint16_t port) {
    assert(p == &live && query_port == 65535 && len == 5 && !memcmp(data, "\xff\xff\xff\xffQ", 5) && ip == 0x01020304 && port == 27015);
    packet_pending = 1;
    return true;
}
int SteamAPI_ISteamGameServer_GetNextOutgoingPacket(void *p, void *data, int cap, uint32_t *ip, uint16_t *port) {
    assert(p == &live && query_port == 65535 && cap == 16384);
    if (!packet_pending) return 0;
    packet_pending = 0;
    memcpy(data, "reply", 5);
    *ip = 0x01020304;
    *port = 27015;
    return 5;
}
bool SteamAPI_ISteamGameServer_RequestUserGroupStatus(void *p, uint64_t user, uint64_t group) {
    assert(p == &live && user == AUTH_PEER && group == AUTH_OWNER);
    server_event_next = 1;
    return true;
}
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
    if (server_event_next) {
        switch (server_event_next++) {
            case 1: *msg = (struct callback_msg){1, 201, &approve_event, sizeof(approve_event)}; break;
            case 2: *msg = (struct callback_msg){1, 202, &deny_event, sizeof(deny_event)}; break;
            case 3: *msg = (struct callback_msg){1, 203, &kick_event, sizeof(kick_event)}; break;
            case 4: *msg = (struct callback_msg){1, 208, &group_event, 18}; break;
            default: server_event_next = 0; return false;
        }
        return true;
    }
    if (socket_callback_pending) {
        uint32_t connection = 700, socket = 314;
        int state = socket_callback_pending == 1 ? 1 : (socket_callback_pending == 2 ? 3 : 4);
        struct identity id;
        SteamAPI_SteamNetworkingIdentity_SetSteamID64(&id, AUTH_PEER);
        memset(socket_event, 0, sizeof(socket_event));
        memcpy(socket_event, &connection, sizeof(connection));
        memcpy(socket_event + 4, &id, sizeof(id));
        memcpy(socket_event + 148, &socket, sizeof(socket));
        memcpy(socket_event + 180, &state, sizeof(state));
        socket_callback_pending = 0;
        *msg = (struct callback_msg){1, 1221, socket_event, sizeof(socket_event)};
        return true;
    }
    if (message_request_pending) {
        message_request_pending = false;
        *msg = (struct callback_msg){1, 1251, &message_request, sizeof(message_request)};
        return true;
    }
    if (p2p_request_pending) {
        p2p_request_pending = false;
        *msg = (struct callback_msg){1, 1202, &p2p_request, sizeof(p2p_request)};
        return true;
    }
    return false;
}
void SteamAPI_ManualDispatch_FreeLastCallback(int pipe) { assert(pipe == 17); }
uint64_t SteamAPI_ISteamGameServer_GetSteamID(void *p) { assert(logged); return UINT64_MAX - 1; }
void SteamGameServer_Shutdown(void) { assert(live && logged && !pending && !socket_open && !child_open && message_releases == 3); live = logged = false; }
