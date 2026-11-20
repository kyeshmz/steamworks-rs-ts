import client = require('./client')

export const enum ChatMemberStateChange {
    Entered,
    Left,
    Disconnected,
    Kicked,
    Banned,
}

export type ChatEntryType =
    | 'Invalid'
    | 'ChatMsg'
    | 'Typing'
    | 'InviteGame'
    | 'Emote'
    | 'LeftConversation'
    | 'Entered'
    | 'WasKicked'
    | 'WasBanned'
    | 'Disconnected'
    | 'HistoricalChat'
    | 'LinkBlocked'

export interface CallbackReturns {
    [client.callback.SteamCallback.SteamInputDeviceConnected]: { device_handle: number | bigint }
    [client.callback.SteamCallback.SteamInputDeviceDisconnected]: { device_handle: number | bigint }
    [client.callback.SteamCallback.SteamInventoryDefinitionUpdate]: null
    [client.callback.SteamCallback.SteamInventoryFullUpdate]: { handle: number }
    [client.callback.SteamCallback.ValidateAuthTicketResponse]: {
        steam_id: bigint
        owner_steam_id: bigint
        response: number
    }
    [client.callback.SteamCallback.ScreenshotRequested]: {}
    [client.callback.SteamCallback.ScreenshotReady]: { handle: number; error: null } | { handle: null; error: string }
    [client.callback.SteamCallback.GameRichPresenceJoinRequested]: {
        friend_steam_id: bigint
        connect: string
    }
    [client.callback.SteamCallback.FriendRichPresenceUpdate]: {
        friend_steam_id: bigint
        app_id: number
    }
    [client.callback.SteamCallback.PersonaStateChange]: {
        steam_id: bigint
        flags: { bits: number }
    }
    [client.callback.SteamCallback.SteamServersConnected]: {}
    [client.callback.SteamCallback.SteamServersDisconnected]: {
        reason: number
    }
    [client.callback.SteamCallback.SteamServerConnectFailure]: {
        reason: number
        still_retrying: boolean
    }
    [client.callback.SteamCallback.LobbyDataUpdate]: {
        lobby: bigint
        member: bigint
        success: boolean
    }
    [client.callback.SteamCallback.LobbyChatUpdate]: {
        lobby: bigint
        user_changed: bigint
        making_change: bigint
        member_state_change: ChatMemberStateChange
    }
    [client.callback.SteamCallback.LobbyChatMsg]: {
        lobby: bigint
        user: bigint
        chat_entry_type: ChatEntryType
        chat_id: number
    }
    [client.callback.SteamCallback.P2PSessionRequest]: {
        remote: bigint
    }
    [client.callback.SteamCallback.P2PSessionConnectFail]: {
        remote: bigint
        error: number
    }
    [client.callback.SteamCallback.GameLobbyJoinRequested]: {
        lobby_steam_id: bigint
        friend_steam_id: bigint
    }
    [client.callback.SteamCallback.MicroTxnAuthorizationResponse]: {
        app_id: number
        order_id: number | bigint
        authorized: boolean
    }
    [client.callback.SteamCallback.GameOverlayActivated]: {
        active: boolean
        userInitiated: boolean
        appId: number
    }
}
