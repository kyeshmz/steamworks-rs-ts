# Authenticate a remote peer

Register one `steam.callback.register(steamworks.SteamCallback.ValidateAuthTicketResponse, handler)` listener before accepting peers. Here `steamworks` is the package and `steam` is its initialized client. Keep the returned handle and route events by `steam_id` to the current pending or active connection. Ignore events for disconnected peers.

Call `steam.auth.beginAuthSession(steamId64, ticket)` with the remote peer's claimed Steam ID as a nonzero unsigned 64-bit `bigint` and their session ticket as a nonempty `Buffer`. Use a session ticket, not a Web API ticket. A thrown error means submission failed. A normal return means Steam accepted the submission, **not** that Steam authorized the peer. Deny access while validation is pending, including when no callback arrives.

The callback delivers `{ steam_id, owner_steam_id, response }`. Both IDs are `bigint`. The license owner can differ from the peer. Only `response === 0` indicates successful ticket validation. Every other `EAuthSessionResponse` value indicates failure. Keep listening after success: Steam can revoke validation later. Remove access immediately on failure.

Call `steam.auth.endAuthSession(steamId64)` for each accepted submission when its peer disconnects, validation fails, or you abandon a pending request. Ending the session does not cancel the issuer's ticket. When shutting down, end remaining sessions before calling `handle.disconnect()`. Disconnecting the callback does not end any session.

On the issuer, overlapping `getSessionTicketWithSteamId`, `getSessionTicketWithIp` and `getAuthTicketForWebApi` requests each wait for their own Steam handle. Replies can arrive out of order. A timeout rejects that request and releases its handle. A successful request returns a `Ticket`; call `ticket.cancel()` when its consumer is finished. Calling `cancel()` again has no effect. Dropping a successful JavaScript ticket does not cancel it on Steam's side.
