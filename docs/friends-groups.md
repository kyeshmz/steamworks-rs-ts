# Named friends groups

Get the `friends` namespace from `require('@kyeshmz/steamworks.ts').init(appId)`. Groups are the named tags the signed-in player created in the Steam friends UI. They are not relationship filters. `friends.getFriends(filter)` lists every friend matching a filter. The functions here list the player's own group names and each group's members.

`friends.getFriendsGroups()` returns `Array<{ id, name }>` in Steam's order. Each `id` is Steam's signed 16-bit group ID. Pass that exact value back to the member read. Do not recompute or reorder it. The array is empty when the account has no groups. Steam allows a user at most 100 groups, so the enumeration reads no more than that.

`friends.getFriendsGroupMembersList(id)` returns that group's members as SteamID64 `bigint` values, in the order Steam returns them. The list is a snapshot taken between the member count and the read. Membership can change between your calls.

Empty and invalid results. A group with no members returns an empty array. An ID that is not one of the user's groups throws `Unknown friends group ID`, including Steam's `-1` invalid marker. The addon uses Steam's documented signal for this, a missing group name, as its validity gate. Take IDs from `getFriendsGroups()` and reuse them unchanged.

The addon sizes the native member buffer from Steam's reported count and refuses counts above 4096 with a thrown error rather than truncating the list.
