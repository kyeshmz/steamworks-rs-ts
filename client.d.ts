/// <reference types="node" />

export interface AchievementIconFetched {
  game_id: bigint
  achievement_name: string
  achieved: boolean
  icon_handle: number
}

export interface AchievementStored {
  game_id: bigint
  achievement_name: string
  current_progress: number
  max_progress: number
}

export interface DeleteItemResult {
  published_file_id: bigint
  error: any
}

export interface DownloadItemResult {
  app_id: number
  published_file_id: bigint
  error: any
}

export interface GlobalStatsReceived {
  game_id: bigint
  error: any
}

export declare function init(appId?: number | undefined | null): void

export declare function isInitialized(): boolean

export interface LobbyCreated {
  result: number
  lobby: bigint
}

export interface LobbyEnter {
  lobby: bigint
  chat_permissions: number
  blocked: boolean
  chat_room_enter_response: string
}

export interface PlayerSteamId {
  steamId64: bigint
  steamId32: string
  accountId: number
}

export interface RelayNetworkStatus {
  availability: number
  ping_measurement_in_progress: boolean
  network_config: number
  any_relay: number
  debug_message: string
}

export interface RemotePlay {
  session: number
}

export declare function restartAppIfNecessary(appId: number): boolean

export declare function runCallbacks(): void

export interface StatsReceived {
  steam_id: bigint
  game_id: bigint
  error: any
}

export interface StatsStored {
  game_id: bigint
  error: any
}

export interface TicketValidation {
  steam_id: bigint
  owner_steam_id: bigint
  response: number
}

export declare namespace achievement {
  export interface AchievementIcon {
    width: number
    height: number
    rgba: Buffer
  }
  export interface AchievementUnlockTime {
    achieved: boolean
    unlockTime: number
  }
  export function activate(achievement: string): boolean
  export function clear(achievement: string): boolean
  export function count(): number
  export function getAchievedPercent(achievement: string): number | null
  export function getAchievementAndUnlockTime(achievement: string): achievement.AchievementUnlockTime | null
  export function getDisplayAttribute(achievement: string, key: string): string | null
  export function getIcon(achievement: string): Buffer | null
  export function getIconRgba(achievement: string): achievement.AchievementIcon | null
  export function isActivated(achievement: string): boolean
  export function names(): Array<string>
  export function requestGlobalPercentages(): Promise<void>
}

export declare namespace apps {
  export function appBuildId(): number
  export function appInstallDir(appId: number): string
  export function appOwner(): PlayerSteamId
  export function availableGameLanguages(): Array<string>
  export function currentBetaName(): string | null
  export function currentGameLanguage(): string
  export interface DlcDownloadProgress {
    bytesDownloaded: bigint
    bytesTotal: bigint
  }
  export interface DlcInfo {
    appId: number
    available: boolean
    name: string
  }
  export function getDlcDownloadProgress(appId: number): apps.DlcDownloadProgress | null
  export function installDlc(appId: number): void
  export function isAppInstalled(appId: number): boolean
  export function isCybercafe(): boolean
  export function isDlcInstalled(appId: number): boolean
  export function isLowViolence(): boolean
  export function isSubscribed(): boolean
  export function isSubscribedApp(appId: number): boolean
  export function isSubscribedFromFreeWeekend(): boolean
  export function isVacBanned(): boolean
  export function launchCommandLine(): string
  export function launchQueryParam(key: string): string
  export function listDlc(): Array<apps.DlcInfo>
  export function uninstallDlc(appId: number): void
}

export declare namespace auth {
  export class Ticket {
    cancel(): void
    getBytes(): Buffer
  }
  export function beginAuthSession(steamId64: bigint, ticket: Buffer): void
  export function endAuthSession(steamId64: bigint): void
  export function getAuthTicketForWebApi(identity: string, timeoutSeconds?: number | undefined | null): Promise<auth.Ticket>
  export function getSessionTicketWithIp(ip: string, timeoutSeconds?: number | undefined | null): Promise<auth.Ticket>
  export function getSessionTicketWithSteamId(steamId64: bigint, timeoutSeconds?: number | undefined | null): Promise<auth.Ticket>
  export function requestEncryptedAppTicket(dataToInclude?: Buffer | undefined | null, timeoutSeconds?: number | undefined | null): Promise<Buffer>
}

export declare namespace callback {
  export class Handle {
    disconnect(): void
  }
  export function register<C extends keyof import('./callbacks').CallbackReturns>(steamCallback: C, handler: (value: import('./callbacks').CallbackReturns[C]) => void): callback.Handle
  export function registerTyped<C extends keyof import('./callbacks').RuntimeCallbackReturns>(steamCallback: C, handler: (value: import('./callbacks').RuntimeCallbackReturns[C]) => void): callback.Handle
  export const enum SteamCallback {
    PersonaStateChange = 0,
    SteamServersConnected = 1,
    SteamServersDisconnected = 2,
    SteamServerConnectFailure = 3,
    LobbyDataUpdate = 4,
    LobbyChatUpdate = 5,
    P2PSessionRequest = 6,
    P2PSessionConnectFail = 7,
    GameLobbyJoinRequested = 8,
    MicroTxnAuthorizationResponse = 9,
    GameOverlayActivated = 10,
    LobbyChatMsg = 11,
    GameRichPresenceJoinRequested = 12,
    FriendRichPresenceUpdate = 13,
    ScreenshotRequested = 14,
    ScreenshotReady = 15,
    ValidateAuthTicketResponse = 16,
    SteamInputDeviceConnected = 17,
    SteamInputDeviceDisconnected = 18,
    SteamInventoryDefinitionUpdate = 19,
    SteamInventoryFullUpdate = 20,
    LobbyCreated = 21,
    LobbyEnter = 22,
    NewUrlLaunchParameters = 23,
    UserStatsReceived = 24,
    UserStatsStored = 25,
    UserAchievementStored = 26,
    UserAchievementIconFetched = 27,
    DownloadItemResult = 28,
    RemotePlayConnected = 29,
    RemotePlayDisconnected = 30,
    RelayNetworkStatus = 31,
    GlobalStatsReceived = 32,
    DeleteItemResult = 33,
  }
}

export declare namespace cloud {
  export class FileInfo {
    name: string
    get size(): bigint
    set size(value: bigint)
  }
  export function beginFileWriteBatch(): boolean
  export interface CloudQuota {
    totalBytes: bigint
    availableBytes: bigint
  }
  export function deleteFile(name: string): boolean
  export function endFileWriteBatch(): boolean
  export function fileExists(name: string): boolean
  export function fileTimestamp(name: string): number
  export function forgetFile(name: string): boolean
  export function getQuota(): cloud.CloudQuota | null
  export function getSyncPlatforms(name: string): number
  export function isEnabledForAccount(): boolean
  export function isEnabledForApp(): boolean
  export function isFilePersisted(name: string): boolean
  export function listFiles(): Array<cloud.FileInfo>
  export function readFile(name: string): string
  export function readFileBytes(name: string): Promise<Buffer>
  export function setEnabledForApp(enabled: boolean): void
  export function setSyncPlatforms(name: string, mask: number): boolean
  export function shareFile(name: string): Promise<bigint>
  export function writeFile(name: string, content: string): boolean
  export function writeFileBytes(name: string, data: Buffer): Promise<void>
}

export declare namespace friends {
  export const enum AvatarSize {
    Small = 0,
    Medium = 1,
    Large = 2,
  }
  export interface CoplayFriend {
    steamId64: bigint
    appId: number
    lastPlayed: number
  }
  export const enum FriendFilter {
    Immediate = 0,
    Blocked = 1,
    FriendshipRequested = 2,
    RequestingFriendship = 3,
    RequestingInfo = 4,
    Ignored = 5,
    IgnoredFriend = 6,
    ClanMember = 7,
    OnGameServer = 8,
    ChatMember = 9,
    All = 10,
  }
  export interface FriendGameInfo {
    appId: number
    lobbyId?: bigint
  }
  export interface FriendsGroup {
    id: number
    name: string
  }
  export function getAvatar(steamId64: bigint, size: friends.AvatarSize): Buffer | null
  export function getCoplayFriends(): Array<friends.CoplayFriend>
  export function getFriends(filter?: friends.FriendFilter | undefined | null): Array<bigint>
  export function getFriendsGroupMembersList(groupId: number): Array<bigint>
  export function getFriendsGroups(): Array<friends.FriendsGroup>
  export function getGamePlayed(steamId64: bigint): friends.FriendGameInfo | null
  export function getNickname(steamId64: bigint): string | null
  export function getPersona(steamId64: bigint): friends.Persona
  export function getPersonaName(steamId64: bigint): string
  export function getRichPresence(steamId64: bigint, key: string): string
  export function getRichPresenceKeys(steamId64: bigint): Array<string>
  export function hasFriend(steamId64: bigint, filter?: friends.FriendFilter | undefined | null): boolean
  export function inviteUserToGame(steamId64: bigint, connect: string): boolean
  export interface Persona {
    name: string
    state: string
    mediumAvatar?: Buffer
  }
  export function requestRichPresence(steamId64: bigint): void
  export function requestUserInformation(steamId64: bigint, nameOnly?: boolean | undefined | null): boolean
  export function setPlayedWith(steamId64: bigint): void
}

export declare namespace game_server {
  export class AuthListener {
    disconnect(): void
  }
  export function acceptHostedConnection(connection: bigint): void
  export function acceptMessageSession(id: bigint): boolean
  export function acceptP2pSession(id: bigint): boolean
  export function beginAuthSession(steamId64: bigint, ticket: Buffer): void
  export function clearAllKeyValues(): void
  export function closeHostedConnection(connection: bigint): boolean
  export function closeHostedListenSocket(): boolean
  export function closeMessageSession(id: bigint): boolean
  export function closeP2pSession(id: bigint): boolean
  export const enum ConnectionState {
    Ready = 'Ready',
    Connecting = 'Connecting',
    Connected = 'Connected',
    Failed = 'Failed',
    Disconnected = 'Disconnected',
  }
  export function diagnostics(): game_server.ServerDiagnostics
  export function endAuthSession(steamId64: bigint): void
  export function getNextOutgoingPacket(): game_server.OutgoingPacket | null
  export function handleIncomingPacket(data: Buffer, address: string, port: number): boolean
  export interface HostedEvent {
    connection: bigint
    state: number
    steamId?: bigint
    endReason: number
  }
  export interface HostedEvents {
    events: Array<game_server.HostedEvent>
    dropped: number
  }
  export function init(options: game_server.ServerOptions): void
  export function logOn(token: string): void
  export function logOnAnonymous(): void
  export interface MessageSessionRequest {
    steamId?: bigint
  }
  export const enum Mode {
    NoAuthentication = 'NoAuthentication',
    Authentication = 'Authentication',
    AuthenticationAndSecure = 'AuthenticationAndSecure',
  }
  export function onMessageSessionRequest(handler: (arg: game_server.MessageSessionRequest) => void): AuthListener
  export function onP2pSessionRequest(handler: (arg: game_server.P2PSessionRequest) => void): AuthListener
  export function onServerEvent(handler: (event: { kind: 'approve'; user: bigint; owner: bigint } | { kind: 'deny'; user: bigint; reason: number; text: string } | { kind: 'kick'; user: bigint; reason: number } | { kind: 'group'; user: bigint; group: bigint; member: boolean; officer: boolean }) => void): game_server.AuthListener
  export function onValidateAuthTicketResponse(handler: (event: { steam_id: bigint; owner_steam_id: bigint; response: number }) => void): game_server.AuthListener
  export function openHostedListenSocket(virtualPort: number): void
  export interface OutgoingPacket {
    address: string
    port: number
    data: Buffer
  }
  export interface P2PSessionRequest {
    steamId: bigint
  }
  export function p2pSessionState(id: bigint): game_server.P2PState | null
  export interface P2PState {
    connectionActive: boolean
    connecting: boolean
    error: number
    usingRelay: boolean
    bytesQueuedForSend: number
    packetsQueuedForSend: number
    remoteIp?: string
    remotePort?: number
  }
  export function receiveHostedMessages(connection: bigint, maxMessages: number): Array<game_server.ServerMessage>
  export function receiveMessages(channel: number, maxMessages: number): Array<game_server.ServerMessage>
  export function requestUserGroupStatus(user: bigint, group: bigint): boolean
  export function runCallbacks(): void
  export function sendHostedMessage(connection: bigint, data: Buffer, reliable: boolean): bigint
  export function sendMessageToUser(id: bigint, data: Buffer, channel: number, reliable: boolean): void
  export interface ServerDiagnostics {
    appId: number
    serverRealTime: number
  }
  export interface ServerEvent {
    kind: string
    user: bigint
    owner?: bigint
    group?: bigint
    reason?: number
    text?: string
    member?: boolean
    officer?: boolean
  }
  export interface ServerListing {
    serverName: string
    mapName: string
    maxPlayers: number
    botPlayers?: number
    passwordProtected?: boolean
  }
  export interface ServerMessage {
    data: Buffer
    steamId?: bigint
  }
  export interface ServerOptions {
    bindIp: string
    gamePort: number
    queryPort: number
    mode: game_server.Mode
    version: string
    product: string
    description: string
    listing: game_server.ServerListing
  }
  export interface ServerStatus {
    state: game_server.ConnectionState
    steamId?: bigint
  }
  export function setAdvertising(active: boolean): void
  export function setGameData(data: string): void
  export function setGameTags(tags: string): void
  export function setKeyValue(key: string, value: string): void
  export function setModDir(directory: string): void
  export function shutdown(): boolean
  export function status(): game_server.ServerStatus
  export function takeHostedEvents(maxEvents: number): game_server.HostedEvents
  export function updateListing(listing: game_server.ServerListing): void
}

export declare namespace input {
  export class Controller {
    activateActionSet(actionSetHandle: bigint): void
    isDigitalActionPressed(actionHandle: bigint): boolean
    getDigitalActionData(action: bigint): DigitalActionData
    getAnalogActionData(action: bigint): AnalogActionData
    getAnalogActionVector(actionHandle: bigint): AnalogActionVector
    getType(): input.InputType
    getHandle(): bigint
    getMotionData(): MotionData
    setLightbarColor(red: number, green: number, blue: number, restoreDefaultLightbarColor?: boolean | undefined | null): void
    getDigitalActionOrigins(actionSet: bigint, action: bigint): Array<number>
    getAnalogActionOrigins(actionSet: bigint, action: bigint): Array<number>
    showBindingPanel(): boolean
    getCurrentActionSet(): bigint
    activateActionSetLayer(layer: bigint): void
    deactivateActionSetLayer(layer: bigint): void
    deactivateAllActionSetLayers(): void
    getActiveActionSetLayers(): Array<bigint>
    triggerVibration(leftSpeed: number, rightSpeed: number): void
    triggerSimpleHapticEvent(location: input.HapticLocation, intensity: number, gainDb: number, otherIntensity: number, otherGainDb: number): void
  }
  export interface AnalogActionData {
    x: number
    y: number
    mode: number
    active: boolean
  }
  export interface AnalogActionVector {
    x: number
    y: number
  }
  export interface DigitalActionData {
    pressed: boolean
    active: boolean
  }
  export function enableDeviceCallbacks(): void
  export function getActionSet(actionSetName: string): bigint
  export function getAnalogAction(actionName: string): bigint
  export function getControllers(): Array<input.Controller>
  export function getDigitalAction(actionName: string): bigint
  export function getGlyphForActionOrigin(origin: number): string
  export function getGlyphPngForActionOrigin(origin: number, size: input.GlyphSize, flags?: number | undefined | null): string
  export function getGlyphSvgForActionOrigin(origin: number, flags?: number | undefined | null): string
  export function getStringForActionOrigin(origin: number): string
  export const enum GlyphSize {
    Small = 0,
    Medium = 1,
    Large = 2,
  }
  export const enum HapticLocation {
    Left = 0,
    Right = 1,
    Both = 2,
  }
  export function init(): void
  export const enum InputType {
    Unknown = 'Unknown',
    SteamController = 'SteamController',
    XBox360Controller = 'XBox360Controller',
    XBoxOneController = 'XBoxOneController',
    GenericGamepad = 'GenericGamepad',
    PS4Controller = 'PS4Controller',
    AppleMFiController = 'AppleMFiController',
    AndroidController = 'AndroidController',
    SwitchJoyConPair = 'SwitchJoyConPair',
    SwitchJoyConSingle = 'SwitchJoyConSingle',
    SwitchProController = 'SwitchProController',
    MobileTouch = 'MobileTouch',
    PS3Controller = 'PS3Controller',
    PS5Controller = 'PS5Controller',
    SteamDeckController = 'SteamDeckController',
  }
  export interface MotionData {
    rotQuatX: number
    rotQuatY: number
    rotQuatZ: number
    rotQuatW: number
    posAccelX: number
    posAccelY: number
    posAccelZ: number
    rotVelX: number
    rotVelY: number
    rotVelZ: number
  }
  export function runFrame(): void
  export function setInputActionManifestFilePath(path: string): boolean
  export function shutdown(): void
}

export declare namespace inventory {
  export function consumeItemSerialized(itemId: bigint, quantity: number): Promise<inventory.SerializedResult>
  export interface ExchangeDestroy {
    itemId: bigint
    quantity: number
  }
  export interface ExchangeGenerate {
    definition: number
    quantity: number
  }
  export function exchangeItems(generate: Array<inventory.ExchangeGenerate>, destroy: Array<inventory.ExchangeDestroy>): Promise<inventory.InventoryResult>
  export function exchangeItemsSerialized(generate: Array<inventory.ExchangeGenerate>, destroy: Array<inventory.ExchangeDestroy>): Promise<inventory.SerializedResult>
  export function getAllItemProperties(): Promise<Array<inventory.ItemProperties>>
  export function getAllItems(): Promise<inventory.InventoryResult>
  export function getItemDefinitionIds(): Array<number> | null
  export function getItemDefinitionProperty(definition: number, name?: string | undefined | null): string | null
  export function getItemProperties(itemIds: Array<bigint>): Promise<Array<inventory.ItemProperties>>
  export function getItemsById(itemIds: Array<bigint>): Promise<inventory.InventoryResult>
  export function getItemsByID(itemIds: Array<bigint>): Promise<inventory.InventoryResult>
  export interface InventoryItem {
    itemId: bigint
    definition: number
    quantity: number
    flags: number
  }
  export interface InventoryResult {
    timestamp: number
    items: Array<inventory.InventoryItem>
  }
  export interface ItemProperties {
    itemId: bigint
    definition: number
    quantity: number
    properties: Record<string, string>
  }
  export function loadItemDefinitions(): boolean
  export function sendItemDropHeartbeat(): void
  export interface SerializedResult {
    timestamp: number
    items: Array<inventory.InventoryItem>
    buffer: Buffer
  }
}

export declare namespace leaderboards {
  export function downloadLeaderboardEntries(name: string, request: leaderboards.LeaderboardDataRequest, rangeStart: number, rangeEnd: number, maxDetails?: number | undefined | null): Promise<Array<leaderboards.LeaderboardEntry>>
  export function findLeaderboard(name: string): Promise<boolean>
  export function findOrCreateLeaderboard(name: string, sortMethod: leaderboards.LeaderboardSortMethod, displayType: leaderboards.LeaderboardDisplayType): Promise<boolean>
  export function getLeaderboardEntryCount(name: string): Promise<number>
  export function getLeaderboardInfo(name: string): Promise<leaderboards.LeaderboardInfo>
  export const enum LeaderboardDataRequest {
    Global = 0,
    GlobalAroundUser = 1,
    Friends = 2,
  }
  export const enum LeaderboardDisplayType {
    Numeric = 0,
    TimeSeconds = 1,
    TimeMilliSeconds = 2,
  }
  export interface LeaderboardEntry {
    steamId: bigint
    globalRank: number
    score: number
    details: Array<number>
  }
  export interface LeaderboardInfo {
    name: string
    entryCount: number
    sortMethod?: leaderboards.LeaderboardSortMethod
    displayType?: leaderboards.LeaderboardDisplayType
  }
  export interface LeaderboardScoreUploaded {
    score: number
    scoreChanged: boolean
    globalRankNew: number
    globalRankPrevious: number
  }
  export const enum LeaderboardSortMethod {
    Ascending = 0,
    Descending = 1,
  }
  export function uploadLeaderboardScore(name: string, sortMethod: leaderboards.LeaderboardSortMethod, displayType: leaderboards.LeaderboardDisplayType, method: leaderboards.UploadScoreMethod, score: number, details?: Array<number> | undefined | null): Promise<leaderboards.LeaderboardScoreUploaded>
  export const enum UploadScoreMethod {
    KeepBest = 0,
    ForceUpdate = 1,
  }
}

export declare namespace localplayer {
  export function clearRichPresence(): void
  export function getIpCountry(): string
  export function getLevel(): number
  export function getName(): string
  export function getSteamId(): PlayerSteamId
  export function getSteamIdDetails(steamId: bigint): localplayer.SteamIdDetails
  export function isLoggedOn(): boolean
  export function setRichPresence(key: string, value?: string | undefined | null): void
  export interface SteamIdDetails {
    universe: number
    accountType: number
    isInvalid: boolean
  }
  export const enum UserHasLicense {
    HasLicense = 0,
    DoesNotHaveLicense = 1,
    NoAuth = 2,
  }
  export function userHasLicenseForApp(steamId: bigint, appId: number): localplayer.UserHasLicense
}

export declare namespace matchmaking {
  export class Lobby {
    get id(): bigint
    set id(value: bigint)
    join(): Promise<matchmaking.Lobby>
    leave(): void
    openInviteDialog(): void
    sendChatMessage(message: Buffer): void
    getChatEntry(chatId: number): Buffer
    getMemberCount(): bigint
    getMemberLimit(): bigint | null
    setMemberLimit(limit: number): boolean
    setOwner(steamId64: bigint): boolean
    setType(lobbyType: matchmaking.LobbyType): boolean
    inviteUser(steamId64: bigint): boolean
    getMembers(): Array<PlayerSteamId>
    getOwner(): PlayerSteamId
    setGameServer(ip: string, port: number, steamId?: bigint | undefined | null): void
    getGameServer(): matchmaking.LobbyGameServer | null
    setMemberData(key: string, value: string): void
    getMemberData(steamId64: bigint, key: string): string | null
    setJoinable(joinable: boolean): boolean
    getData(key: string): string | null
    setData(key: string, value: string): boolean
    deleteData(key: string): boolean
    getFullData(): Record<string, string>
    mergeFullData(data: Record<string, string>): boolean
  }
  export function createLobby(lobbyType: matchmaking.LobbyType, maxMembers: number): Promise<matchmaking.Lobby>
  export function getLobbies(filter?: matchmaking.LobbyListFilter | undefined | null): Promise<Array<matchmaking.Lobby>>
  export function joinLobby(lobbyId: bigint): Promise<matchmaking.Lobby>
  export const enum LobbyComparison {
    Equal = 0,
    NotEqual = 1,
    GreaterThan = 2,
    GreaterThanOrEqual = 3,
    LessThan = 4,
    LessThanOrEqual = 5,
  }
  export const enum LobbyDistance {
    Close = 0,
    Default = 1,
    Far = 2,
    Worldwide = 3,
  }
  export interface LobbyGameServer {
    ip: string
    port: number
    steamId?: bigint
  }
  export interface LobbyListFilter {
    stringFilters?: Array<matchmaking.LobbyStringFilter>
    numberFilters?: Array<matchmaking.LobbyNumberFilter>
    nearValueFilters?: Array<matchmaking.LobbyNearFilter>
    slotsAvailable?: number
    distance?: matchmaking.LobbyDistance
    count?: number
  }
  export interface LobbyNearFilter {
    key: string
    value: number
  }
  export interface LobbyNumberFilter {
    key: string
    value: number
    comparison: matchmaking.LobbyComparison
  }
  export interface LobbyStringFilter {
    key: string
    value: string
    comparison?: matchmaking.LobbyComparison
  }
  export const enum LobbyType {
    Private = 0,
    FriendsOnly = 1,
    Public = 2,
    Invisible = 3,
  }
}

export declare namespace matchmaking_servers {
  export class ServerList {
    isRefreshing(): boolean
    count(): number
    getServer(index: number): matchmaking_servers.ServerInfo | null
    cancel(): boolean
    release(): boolean
  }
  export function requestList(appId: number, kind: 'internet' | 'lan' | 'friends' | 'favorites' | 'history' | 'spectator', filters?: Array<matchmaking_servers.ServerFilter> | undefined | null): matchmaking_servers.ServerList
  export interface ServerFilter {
    key: string
    value: string
  }
  export interface ServerInfo {
    appId: number
    steamId: bigint
    address: string
    queryPort: number
    connectionPort: number
    pingMs: number
    players: number
    maxPlayers: number
    botPlayers: number
    version: number
    lastTimePlayed: number
    successfulResponse: boolean
    doNotRefresh: boolean
    password: boolean
    secure: boolean
    name: string
    map: string
    gameDir: string
    description: string
    tags: string
  }
}

export declare namespace networking {
  export function acceptP2PSession(steamId64: bigint): void
  export function closeP2PSession(steamId64: bigint): void
  export function getP2PSessionState(steamId64: bigint): networking.P2PSessionState | null
  export function isP2PPacketAvailable(): number
  export function isP2PPacketAvailableOnChannel(channel: number): number
  export interface P2PPacket {
    data: Buffer
    size: number
    steamId: PlayerSteamId
  }
  export interface P2PSessionState {
    connectionActive: boolean
    connecting: boolean
    error: number
    usingRelay: boolean
    bytesQueuedForSend: number
    packetsQueuedForSend: number
    remoteIp?: string
    remotePort?: number
  }
  export function readP2PPacket(size: number): networking.P2PPacket
  export function readP2PPacketFromChannel(size: number, channel: number): networking.P2PPacket
  export function sendP2PPacket(steamId64: bigint, sendType: networking.SendType, data: Buffer): boolean
  export function sendP2PPacketOnChannel(steamId64: bigint, sendType: networking.SendType, data: Buffer, channel: number): boolean
  export const enum SendType {
    Unreliable = 0,
    UnreliableNoDelay = 1,
    Reliable = 2,
    ReliableWithBuffering = 3,
  }
  export function tryAcceptP2PSession(steamId64: bigint): boolean
  export function tryCloseP2PSession(steamId64: bigint): boolean
}

export declare namespace networking_messages {
  export function acceptSessionWithUser(steamId64: bigint): boolean
  export function closeChannelWithUser(steamId64: bigint, channel: number): boolean
  export function closeSessionWithUser(steamId64: bigint): boolean
  export function getSessionConnectionInfo(steamId64: bigint): networking_messages.SessionConnectionInfo | null
  export interface Message {
    data: Buffer
    steamId?: PlayerSteamId
  }
  export const enum MessageSendType {
    ReliableNoNagle = 0,
    Reliable = 1,
    Unreliable = 2,
    UnreliableNoDelay = 3,
  }
  export function receiveMessagesOnChannel(channel?: number | undefined | null, maxMessages?: number | undefined | null): Array<networking_messages.Message>
  export function sendMessageToUser(steamId64: bigint, data: Buffer, channel?: number | undefined | null, sendType?: networking_messages.MessageSendType | undefined | null): void
  export interface SessionConnectionInfo {
    state: number
    steamId64?: bigint
    listenSocket: number
    endReason: number
    ping: number
    qualityLocal: number
    qualityRemote: number
  }
  export interface SessionFailure {
    steamId?: PlayerSteamId
  }
  export interface SessionRequest {
    steamId?: PlayerSteamId
  }
  export function setSessionFailedHandler(handler?: (failure: SessionFailure) => void): void
  export function setSessionRequestHandler(handler?: (request: SessionRequest) => void): void
}

export declare namespace networking_sockets {
  export function acceptConnection(connection: number): void
  export function authenticationStatus(): networking_sockets.AuthenticationStatus
  export interface AuthenticationStatus {
    availability: number
    debugMessage: string
  }
  export function closeConnection(connection: number, linger?: boolean | undefined | null): boolean
  export function closeListenSocket(socket: number): boolean
  export function configureConnectionLanes(connection: number, lanes: Array<networking_sockets.LaneConfig>): void
  export function connectByIpAddress(address: string, port: number): number
  export function connectionInfo(connection: number): networking_sockets.ConnectionInfo | null
  export interface ConnectionInfo {
    connection: number
    listenSocket: number
    steamId64?: bigint
    state: number
    endReason: number
  }
  export function connectionName(connection: number): string | null
  export function connectionUserData(connection: number): bigint | null
  export function connectP2P(steamId64: bigint, virtualPort: number): number
  export function createListenSocketIp(port: number, address?: string | undefined | null): number
  export function createListenSocketP2P(virtualPort: number): number
  export function createPollGroup(): number
  export function destroyPollGroup(group: number): boolean
  export function detailedRelayNetworkStatus(): networking_sockets.RelayNetworkStatus
  export function flushMessages(connection: number): void
  export function initAuthentication(): number
  export function initRelayNetworkAccess(): void
  export interface LaneConfig {
    priority: number
    weight: number
  }
  export interface LaneStatus {
    pendingUnreliable: number
    pendingReliable: number
    sentUnackedReliable: number
    queueTimeMicroseconds: bigint
  }
  export interface OutgoingMessage {
    connection: number
    data: Buffer
    reliable?: boolean
    lane?: number
  }
  export interface PollGroupMessage {
    connection: number
    data: Buffer
  }
  export function realtimeConnectionStatus(connection: number, lanes?: number | undefined | null): networking_sockets.RealtimeStatus
  export interface RealtimeStatus {
    state: number
    ping: number
    qualityLocal: number
    qualityRemote: number
    outPacketsPerSecond: number
    outBytesPerSecond: number
    inPacketsPerSecond: number
    inBytesPerSecond: number
    sendRateBytesPerSecond: number
    pendingUnreliable: number
    pendingReliable: number
    sentUnackedReliable: number
    queueTimeMicroseconds: bigint
    lanes: Array<networking_sockets.LaneStatus>
  }
  export function receiveMessages(connection: number, maxMessages?: number | undefined | null): Array<Buffer>
  export function receiveMessagesOnPollGroup(group: number, maxMessages?: number | undefined | null): Array<networking_sockets.PollGroupMessage>
  export interface RelayNetworkStatus {
    availability: string
    pingMeasurementInProgress: boolean
    networkConfig: string
    anyRelay: string
    debugMessage: string
  }
  export function relayStatus(): string
  export function sendMessage(connection: number, data: Buffer, reliable?: boolean | undefined | null): bigint
  export interface SendMessageResult {
    messageNumber?: bigint
    error?: bigint
  }
  export function sendMessages(messages: Array<networking_sockets.OutgoingMessage>): Array<networking_sockets.SendMessageResult>
  export function setConnectionName(connection: number, name: string): void
  export function setConnectionPollGroup(connection: number, group: number): boolean
  export function setConnectionStatusHandler(handler?: (info: ConnectionInfo) => void): void
  export function setConnectionUserData(connection: number, value: bigint): boolean
}

export declare namespace overlay {
  export function activateDialog(dialog: overlay.Dialog): void
  export function activateDialogToUser(dialog: overlay.Dialog, steamId64: bigint): void
  export function activateInviteDialog(lobbyId: bigint): void
  export function activateInviteDialogConnectString(connect: string): void
  export function activateToStore(appId: number, flag: overlay.StoreFlag): void
  export function activateToWebPage(url: string): void
  export const enum Dialog {
    Friends = 0,
    Community = 1,
    Players = 2,
    Settings = 3,
    OfficialGameGroup = 4,
    Stats = 5,
    Achievements = 6,
  }
  export const enum NotificationPosition {
    TopLeft = 0,
    TopRight = 1,
    BottomLeft = 2,
    BottomRight = 3,
  }
  export function setNotificationPosition(position: overlay.NotificationPosition): void
  export const enum StoreFlag {
    None = 0,
    AddToCart = 1,
    AddToCartAndShow = 2,
  }
}

export declare namespace remote_play {
  export function invite(friend: bigint): boolean
  export interface RemotePlayResolution {
    width: number
    height: number
  }
  export interface RemotePlaySession {
    id: number
    user: bigint
    clientName: string
    clientFormFactor?: string
    clientResolution?: remote_play.RemotePlayResolution
  }
  export function session(id: number): remote_play.RemotePlaySession | null
  export function sessions(): Array<number>
}

export declare namespace runtime {
  export function pumpStats(reset?: boolean | undefined | null): runtime.PumpStats
  export interface PumpStats {
    runs: number
    gaps: number
    meanGapUs: number
    maxGapUs: number
    lastGapUs: number
    maxRunUs: number
  }
}

export declare namespace screenshots {
  export function addScreenshotToLibrary(filename: string, thumbnailFilename: string | null, width: number, height: number): number
  export function hookScreenshots(enabled: boolean): void
  export function isScreenshotsHooked(): boolean
  export function setLocation(screenshot: number, location: string): boolean
  export function tagPublishedFile(screenshot: number, fileId: bigint): boolean
  export function tagUser(screenshot: number, steamId64: bigint): boolean
  export function triggerScreenshot(): void
  export function writeScreenshot(data: Buffer, width: number, height: number): number
}

export declare namespace stats {
  export function getFloat(name: string): number | null
  export function getGlobalFloat(name: string): number | null
  export function getGlobalFloatHistory(name: string, maxDays: number): Array<number> | null
  export function getGlobalInt(name: string): bigint | null
  export function getGlobalIntHistory(name: string, maxDays: number): Array<bigint> | null
  export function getInt(name: string): number | null
  export function getUserAchievement(steamId: bigint, name: string): boolean | null
  export function getUserStatFloat(steamId: bigint, name: string): number | null
  export function getUserStatInt(steamId: bigint, name: string): number | null
  export function requestGlobalStats(historyDays: number): Promise<bigint>
  export function requestUserStats(steamId: bigint): void
  export function resetAll(achievementsToo: boolean): boolean
  export function setFloat(name: string, value: number): boolean
  export function setInt(name: string, value: number): boolean
  export function store(): boolean
}

export declare namespace timeline {
  export function addTimelineEvent(icon: string, title: string, description: string, priority: number, startOffsetSeconds: number, durationSeconds: number, clipPriority: timeline.ClipPriority): void
  export function clearTimelineStateDescription(seconds: number): void
  export const enum ClipPriority {
    None = 0,
    Standard = 1,
    Featured = 2,
  }
  export const enum GameMode {
    Playing = 0,
    Staging = 1,
    Menus = 2,
    LoadingScreen = 3,
  }
  export function setTimelineGameMode(mode: timeline.GameMode): void
  export function setTimelineStateDescription(description: string, seconds: number): void
}

export declare namespace utils {
  export const enum FloatingGamepadTextInputMode {
    SingleLine = 0,
    MultipleLines = 1,
    Email = 2,
    Numeric = 3,
  }
  export const enum GamepadTextInputLineMode {
    SingleLine = 0,
    MultipleLines = 1,
  }
  export const enum GamepadTextInputMode {
    Normal = 0,
    Password = 1,
  }
  export function getAppId(): number
  export function getServerRealTime(): number
  export function isOverlayEnabled(): boolean
  export function isSteamInBigPictureMode(): boolean
  export function isSteamRunningOnSteamDeck(): boolean
  export function setWarningHandler(handler?: (warning: SteamWarning) => void): void
  export function showFloatingGamepadTextInput(keyboardMode: utils.FloatingGamepadTextInputMode, x: number, y: number, width: number, height: number): Promise<boolean>
  export function showGamepadTextInput(inputMode: utils.GamepadTextInputMode, inputLineMode: utils.GamepadTextInputLineMode, description: string, maxCharacters: number, existingText?: string | undefined | null): Promise<string | null>
  export interface SteamWarning {
    severity: number
    message: string
  }
  export function uiLanguage(): string
}

export declare namespace voice {
  export function decompressVoice(compressed: Buffer, sampleRate: number): Buffer
  export function getAvailableVoice(): number
  export function getVoice(): Buffer
  export function getVoiceOptimalSampleRate(): number
  export function startVoiceRecording(): void
  export function stopVoiceRecording(): void
}

export declare namespace workshop {
  export interface AppIDs {
    creator?: number
    consumer?: number
  }
  export const enum ContentDescriptor {
    NudityOrSexualContent = 0,
    FrequentViolenceOrGore = 1,
    AdultOnlySexualContent = 2,
    GratuitousSexualContent = 3,
    AnyMatureContent = 4,
  }
  export function createItem(appId?: number | undefined | null, fileType?: workshop.FileType | undefined | null): Promise<workshop.UgcResult>
  export function deleteItem(itemId: bigint): Promise<void>
  export function download(itemId: bigint, highPriority: boolean): boolean
  export function downloadInfo(itemId: bigint): workshop.DownloadInfo | null
  export interface DownloadInfo {
    current: bigint
    total: bigint
  }
  export const enum FileType {
    Community = 0,
    Microtransaction = 1,
    Collection = 2,
    Art = 3,
    Video = 4,
    Screenshot = 5,
    Game = 6,
    Software = 7,
    Concept = 8,
    WebGuide = 9,
    IntegratedGuide = 10,
    Merch = 11,
    ControllerBinding = 12,
    SteamworksAccessInvite = 13,
    SteamVideo = 14,
    GameManagedItem = 15,
  }
  export function getAllItemIds(page: number, queryType: workshop.UGCQueryType, itemType: workshop.UGCType, creatorAppId: number, consumerAppId: number, queryConfig?: workshop.WorkshopItemQueryConfig | undefined | null): Promise<Array<bigint>>
  export function getAllItems(page: number, queryType: workshop.UGCQueryType, itemType: workshop.UGCType, creatorAppId: number, consumerAppId: number, queryConfig?: workshop.WorkshopItemQueryConfig | undefined | null): Promise<workshop.WorkshopPaginatedResult>
  export function getAllItemTotal(page: number, queryType: workshop.UGCQueryType, itemType: workshop.UGCType, creatorAppId: number, consumerAppId: number, queryConfig?: workshop.WorkshopItemQueryConfig | undefined | null): Promise<number>
  export function getItem(item: bigint, queryConfig?: workshop.WorkshopItemQueryConfig | undefined | null): Promise<workshop.WorkshopItem | null>
  export function getItems(items: Array<bigint>, queryConfig?: workshop.WorkshopItemQueryConfig | undefined | null): Promise<workshop.WorkshopItemsResult>
  export function getSubscribedItems(includeLocallyDisabled?: boolean | undefined | null): Array<bigint>
  export function getUserItemIds(page: number, accountId: number, listType: workshop.UserListType, itemType: workshop.UGCType, sortOrder: workshop.UserListOrder, appIds: workshop.AppIDs, queryConfig?: workshop.WorkshopItemQueryConfig | undefined | null): Promise<Array<bigint>>
  export function getUserItems(page: number, accountId: number, listType: workshop.UserListType, itemType: workshop.UGCType, sortOrder: workshop.UserListOrder, appIds: workshop.AppIDs, queryConfig?: workshop.WorkshopItemQueryConfig | undefined | null): Promise<workshop.WorkshopPaginatedResult>
  export function installInfo(itemId: bigint): workshop.InstallInfo | null
  export interface InstallInfo {
    folder: string
    sizeOnDisk: bigint
    timestamp: number
  }
  export interface KeyValueTag {
    key: string
    value: string
  }
  export function startPlaytimeTracking(itemIds: Array<bigint>): Promise<void>
  export function state(itemId: bigint): number
  export function stopPlaytimeTracking(itemIds: Array<bigint>): Promise<void>
  export function stopPlaytimeTrackingForAllItems(): Promise<void>
  export function subscribe(itemId: bigint): Promise<void>
  export function suspendDownloads(suspend: boolean): void
  export const enum UgcItemVisibility {
    Public = 0,
    FriendsOnly = 1,
    Private = 2,
    Unlisted = 3,
  }
  export const enum UGCQueryType {
    RankedByVote = 0,
    RankedByPublicationDate = 1,
    AcceptedForGameRankedByAcceptanceDate = 2,
    RankedByTrend = 3,
    FavoritedByFriendsRankedByPublicationDate = 4,
    CreatedByFriendsRankedByPublicationDate = 5,
    RankedByNumTimesReported = 6,
    CreatedByFollowedUsersRankedByPublicationDate = 7,
    NotYetRated = 8,
    RankedByTotalVotesAsc = 9,
    RankedByVotesUp = 10,
    RankedByTextSearch = 11,
    RankedByTotalUniqueSubscriptions = 12,
    RankedByPlaytimeTrend = 13,
    RankedByTotalPlaytime = 14,
    RankedByAveragePlaytimeTrend = 15,
    RankedByLifetimeAveragePlaytime = 16,
    RankedByPlaytimeSessionsTrend = 17,
    RankedByLifetimePlaytimeSessions = 18,
    RankedByLastUpdatedDate = 19,
  }
  export interface UgcResult {
    itemId: bigint
    needsToAcceptAgreement: boolean
  }
  export const enum UGCType {
    Items = 0,
    ItemsMtx = 1,
    ItemsReadyToUse = 2,
    Collections = 3,
    Artwork = 4,
    Videos = 5,
    Screenshots = 6,
    AllGuides = 7,
    WebGuides = 8,
    IntegratedGuides = 9,
    UsableInGame = 10,
    ControllerBindings = 11,
    GameManagedItems = 12,
    All = 13,
  }
  export interface UgcUpdate {
    title?: string
    description?: string
    language?: string
    changeNote?: string
    previewPath?: string
    contentPath?: string
    tags?: Array<string>
    visibility?: workshop.UgcItemVisibility
    metadata?: string
    keyValueTags?: Array<workshop.KeyValueTag>
    removeKeyValueTags?: Array<string>
    removeAllKeyValueTags?: boolean
    allowAdminTags?: boolean
    contentDescriptors?: Array<workshop.ContentDescriptor>
    removeContentDescriptors?: Array<workshop.ContentDescriptor>
  }
  export function unsubscribe(itemId: bigint): Promise<void>
  export function updateItem(itemId: bigint, updateDetails: workshop.UgcUpdate, appId?: number | undefined | null): Promise<workshop.UgcResult>
  export function updateItemWithCallback(itemId: bigint, updateDetails: workshop.UgcUpdate, appId: number | undefined | null, successCallback: (data: UgcResult) => void, errorCallback: (err: any) => void, progressCallback?: (data: UpdateProgress) => void, progressCallbackIntervalMs?: number | undefined | null): void
  export interface UpdateProgress {
    status: workshop.UpdateStatus
    progress: bigint
    total: bigint
  }
  export const enum UpdateStatus {
    Invalid = 0,
    PreparingConfig = 1,
    PreparingContent = 2,
    UploadingContent = 3,
    UploadingPreviewFile = 4,
    CommittingChanges = 5,
  }
  export const enum UserListOrder {
    CreationOrderAsc = 0,
    CreationOrderDesc = 1,
    TitleAsc = 2,
    LastUpdatedDesc = 3,
    SubscriptionDateDesc = 4,
    VoteScoreDesc = 5,
    ForModeration = 6,
  }
  export const enum UserListType {
    Published = 0,
    VotedOn = 1,
    VotedUp = 2,
    VotedDown = 3,
    Favorited = 4,
    Subscribed = 5,
    UsedOrPlayed = 6,
    Followed = 7,
  }
  export interface WorkshopItem {
    publishedFileId: bigint
    creatorAppId?: number
    consumerAppId?: number
    title: string
    description: string
    owner: PlayerSteamId
    timeCreated: number
    timeUpdated: number
    timeAddedToUserList: number
    visibility: workshop.UgcItemVisibility
    banned: boolean
    acceptedForUse: boolean
    tags: Array<string>
    tagsTruncated: boolean
    url: string
    numUpvotes: number
    numDownvotes: number
    numChildren: number
    previewUrl?: string
    metadata?: string
    keyValueTags?: Array<workshop.KeyValueTag>
    children?: Array<bigint>
    statistics: workshop.WorkshopItemStatistic
  }
  export interface WorkshopItemQueryConfig {
    cachedResponseMaxAge?: number
    includeMetadata?: boolean
    returnChildren?: boolean
    includeKeyValueTags?: boolean
    requiredKeyValueTags?: Array<workshop.KeyValueTag>
    cloudFileNameFilter?: string
    includeLongDescription?: boolean
    includeAdditionalPreviews?: boolean
    onlyIds?: boolean
    onlyTotal?: boolean
    language?: string
    matchAnyTag?: boolean
    requiredTags?: Array<string>
    excludedTags?: Array<string>
    searchText?: string
    rankedByTrendDays?: number
  }
  export interface WorkshopItemsResult {
    items: Array<workshop.WorkshopItem | undefined | null>
    wasCached: boolean
  }
  export interface WorkshopItemStatistic {
    numSubscriptions?: bigint
    numFavorites?: bigint
    numFollowers?: bigint
    numUniqueSubscriptions?: bigint
    numUniqueFavorites?: bigint
    numUniqueFollowers?: bigint
    numUniqueWebsiteViews?: bigint
    reportScore?: bigint
    numSecondsPlayed?: bigint
    numPlaytimeSessions?: bigint
    numComments?: bigint
    numSecondsPlayedDuringTimePeriod?: bigint
    numPlaytimeSessionsDuringTimePeriod?: bigint
  }
  export interface WorkshopPaginatedResult {
    items: Array<workshop.WorkshopItem | undefined | null>
    returnedResults: number
    totalResults: number
    wasCached: boolean
  }
}
