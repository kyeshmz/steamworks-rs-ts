# Peer messages

`closeChannelWithUser(peerId, channel)` closes a single channel without closing the peer's other channels. `closeSessionWithUser(peerId)` closes the entire session.

`client.networking_messages` wraps `ISteamNetworkingMessages`. It is separate from `client.networking`, which uses the legacy P2P transport. Both peers must use the same transport and App ID.

After matching players, obtain the other user's SteamID64 from a trusted source such as your lobby membership list. The receiver can accept a request without sending anything. Register the handler before inviting a peer to connect. The wrapper never accepts a request merely because it arrived; sending to a peer does implicitly accept a session from that peer.

```js
function startPeer(client, peerId, initiate) {
    const net = client.networking_messages
    const channel = 1
    net.setSessionRequestHandler(({ steamId }) => {
        if (steamId?.steamId64 === peerId) net.acceptSessionWithUser(peerId)
    })
    net.setSessionFailedHandler(({ steamId }) => {
        console.error('Steam session failed', steamId?.steamId64)
    })
    if (initiate) net.sendMessageToUser(peerId, Buffer.from('hello'), channel)
    const timer = setInterval(() => {
        for (const { data, steamId } of net.receiveMessagesOnChannel(channel)) {
            if (steamId?.steamId64 === peerId) console.log(data.toString())
        }
    }, 30)
    return () => {
        clearInterval(timer)
        net.setSessionRequestHandler()
        net.setSessionFailedHandler()
        net.closeSessionWithUser(peerId)
    }
}
```

- `sendMessageToUser(steamId64, buffer, channel?, sendType?)` sends up to 512 KiB. Pass a `bigint` individual user ID, not a decimal string or `number`. Channel defaults to `0`. `MessageSendType.ReliableNoNagle` is the default. Other modes are `Reliable`, `Unreliable` and `UnreliableNoDelay`. Success means Steam queued the message, not that the peer received it. A send error throws. A new send may restart a broken session.
- `receiveMessagesOnChannel(channel?, maxMessages?)` returns up to 10 messages by default, or `[]` if the channel is empty. Channels are integers from 0 to 2147483647. Batch size is an integer from 1 to 32. Each message owns its Buffer. `steamId` is absent for non-Steam identities. Reliable ordering applies only within one peer and channel.
- `setSessionRequestHandler(handler?)` installs one request listener per initialized client. Replace or clear it just like the failure listener. Ignore unexpected IDs and requests without a user identity. Validate the peer against your current access policy when the handler runs.
- `acceptSessionWithUser(steamId64)` and `closeSessionWithUser(steamId64)` return Steam's boolean result. Closing releases the session across all channels; it does not block the peer from requesting another session.
- `setSessionFailedHandler(handler?)` sets one failure listener per initialized client, not per channel or peer. A new listener replaces the old one. Omit the argument to clear it. Calls already queued for JavaScript may still run. Steam may not know the failed peer's identity. The package's `init` runs Steam callbacks automatically; keep polling your channels separately. Stop your interval with `clearInterval`.

Legacy `P2PSessionRequest` and `P2PSessionConnectFail` events do not describe this transport. Use the handlers on `networking_messages` instead.
