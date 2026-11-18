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
	return { friends, board };
}
