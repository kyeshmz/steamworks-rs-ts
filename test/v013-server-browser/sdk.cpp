#include "isteammatchmaking.h"
#include <cassert>
#include <atomic>
#include <chrono>
#include <cstring>
#include <thread>

static int browser;
static int outstanding;
static std::atomic<int> active_calls{0};

struct Guard {
    Guard() { assert(active_calls.fetch_add(1) == 0); }
    ~Guard() { assert(active_calls.fetch_sub(1) == 1); }
};

struct Request {
    ISteamMatchmakingServerListResponse *response;
    gameserveritem_t item;
    int polls;
};

static Request *create(uint32 app, MatchMakingKeyValuePair_t **filters, uint32 count,
                       ISteamMatchmakingServerListResponse *response) {
    Guard guard;
    assert(app == 3104020);
    assert(response);
    if (count) {
        assert(count == 2 && filters && filters[0] && filters[1]);
        assert(std::strcmp(filters[0]->m_szKey, "map") == 0);
        assert(std::strcmp(filters[0]->m_szValue, "island") == 0);
        assert(std::strcmp(filters[1]->m_szKey, "secure") == 0);
        assert(std::strcmp(filters[1]->m_szValue, "1") == 0);
    }
    Request *r = new Request{};
    ++outstanding;
    r->response = response;
    r->item.m_nAppID = app;
    r->item.m_steamID.SetFromUint64(76561198012345678ULL);
    r->item.m_NetAdr.Init(0xc0000201, 27015, 27016);
    r->item.m_nPing = -1;
    r->item.m_nPlayers = 4;
    r->item.m_nMaxPlayers = 12;
    r->item.m_nBotPlayers = 1;
    r->item.m_nServerVersion = outstanding;
    r->item.m_bHadSuccessfulResponse = true;
    r->item.m_bSecure = true;
    r->item.SetName("Fixture");
    std::strcpy(r->item.m_szMap, "island");
    std::strcpy(r->item.m_szGameDir, "game");
    std::strcpy(r->item.m_szGameDescription, "Fixture game");
    std::strcpy(r->item.m_szGameTags, "test");
    response->ServerResponded(r, 0);
    response->ServerFailedToRespond(r, 1);
    return r;
}

extern "C" {
void Fixture_Original_RunFrame(int);
void SteamAPI_ManualDispatch_RunFrame(int pipe) {
    Guard guard;
    std::this_thread::sleep_for(std::chrono::milliseconds(1));
    Fixture_Original_RunFrame(pipe);
}

ISteamMatchmakingServers *SteamAPI_SteamMatchmakingServers_v002() {
    return reinterpret_cast<ISteamMatchmakingServers *>(&browser);
}

#define LIST(name) \
HServerListRequest SteamAPI_ISteamMatchmakingServers_Request##name##ServerList( \
    ISteamMatchmakingServers *self, uint32 app, MatchMakingKeyValuePair_t **filters, \
    uint32 count, ISteamMatchmakingServerListResponse *response) { \
    assert(self == reinterpret_cast<ISteamMatchmakingServers *>(&browser)); \
    return create(app, filters, count, response); \
}
LIST(Internet)
LIST(Friends)
LIST(Favorites)
LIST(History)
LIST(Spectator)

HServerListRequest SteamAPI_ISteamMatchmakingServers_RequestLANServerList(
    ISteamMatchmakingServers *self, uint32 app, ISteamMatchmakingServerListResponse *response) {
    assert(self == reinterpret_cast<ISteamMatchmakingServers *>(&browser));
    return create(app, nullptr, 0, response);
}

bool SteamAPI_ISteamMatchmakingServers_IsRefreshing(ISteamMatchmakingServers *, HServerListRequest handle) {
    Request *r = static_cast<Request *>(handle);
    if (++r->polls == 1) return true;
    if (r->polls == 2) r->response->RefreshComplete(r, eServerResponded);
    return false;
}

int SteamAPI_ISteamMatchmakingServers_GetServerCount(ISteamMatchmakingServers *, HServerListRequest) {
    return 2;
}

gameserveritem_t *SteamAPI_ISteamMatchmakingServers_GetServerDetails(
    ISteamMatchmakingServers *, HServerListRequest handle, int index) {
    assert(index >= 0 && index < 2);
    return index == 0 ? &static_cast<Request *>(handle)->item : nullptr;
}

void SteamAPI_ISteamMatchmakingServers_ReleaseRequest(ISteamMatchmakingServers *, HServerListRequest handle) {
    Guard guard;
    assert(outstanding > 0);
    --outstanding;
    delete static_cast<Request *>(handle);
}

int fixture_outstanding() { return outstanding; }
}
