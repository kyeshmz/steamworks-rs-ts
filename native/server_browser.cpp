#include "isteammatchmaking.h"

class ServerListResponse final : public ISteamMatchmakingServerListResponse {
public:
    void ServerResponded(HServerListRequest, int) override {}
    void ServerFailedToRespond(HServerListRequest, int) override {}
    void RefreshComplete(HServerListRequest, EMatchMakingServerResponse) override {}
};

extern "C" ISteamMatchmakingServerListResponse* steamworksjs_server_list_response() {
    static ServerListResponse response;
    return &response;
}
