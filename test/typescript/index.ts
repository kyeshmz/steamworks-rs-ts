import * as steamworks from "@kyeshmz/steamworks.ts";

export function listenForServerValidation(check: (peer: bigint, status: number) => void) {
	return steamworks.gameServer.onValidateAuthTicketResponse(event => check(event.steam_id, event.response));
}

export async function exchangeForReceipt(client: ReturnType<typeof steamworks.init>, itemId: bigint): Promise<Buffer> {
	const result = await client.inventory.exchangeItemsSerialized(
		[{ definition: 1002, quantity: 1 }], [{ itemId, quantity: 3 }]
	);
	return result.buffer;
}

export default function main() {
	const client = steamworks.init(480);
	console.log(client.localplayer.getName())
	client.callback.register(steamworks.SteamCallback.GameOverlayActivated, ({ active }) => {
		const overlayVisible: boolean = active;
		console.log(overlayVisible);
	});
	const friends: bigint[] = client.friends.getFriends();
	const groupMembers: bigint[][] = client.friends.getFriendsGroups()
		.map(group => client.friends.getFriendsGroupMembersList(group.id));
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
	return { friends, groupMembers, board, itemIds };
}
