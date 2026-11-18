import * as steamworks from "@kyeshmz/steamworks.ts";

export default function main() {
	const client = steamworks.init(480);
	console.log(client.localplayer.getName())
	client.callback.register(steamworks.SteamCallback.GameOverlayActivated, ({ active }) => {
		const overlayVisible: boolean = active;
		console.log(overlayVisible);
	});
	const friends: bigint[] = client.friends.getFriends();
	const board: Promise<boolean> = client.leaderboards.findLeaderboard("high-scores");
	client.callback.register(steamworks.SteamCallback.GameRichPresenceJoinRequested, event => {
		const invite: [bigint, string] = [event.friend_steam_id, event.connect];
		console.log(invite);
	});
	client.callback.register(steamworks.SteamCallback.ScreenshotReady, event => {
		if (event.handle !== null) console.log(event.handle.toFixed());
	});
	client.networking_sockets.setConnectionStatusHandler(info => {
		const peer: bigint | undefined = info.steamId64;
		console.log(peer, info.state);
	});
	const items = client.inventory.getAllItems().then(result => result.items.map(item => item.itemId));
	const itemIds: Promise<bigint[]> = items;
	return { friends, board, itemIds };
}
