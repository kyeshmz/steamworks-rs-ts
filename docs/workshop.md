# Workshop details

`updateItem` and `updateItemWithCallback` accept the same update object. Set `metadata` to an arbitrary string of up to 4999 UTF-8 bytes. The pinned SDK's 5000-byte read buffer needs one byte for the terminator.

```js
await client.workshop.updateItem(itemId, {
  metadata: JSON.stringify({ format: 2 }),
  removeKeyValueTags: ['mode'],
  keyValueTags: [
    { key: 'mode', value: 'solo' },
    { key: 'mode', value: 'co-op' },
  ],
})
```

Removals run before additions. Use `removeAllKeyValueTags: true` to replace the entire set. Keys accept ASCII letters, digits, and underscores. Keys and values can contain at most 255 bytes each. An update can add at most 100 pairs. `allowAdminTags` applies only to ordinary `tags`, not key/value tags. Use `contentDescriptors` and `removeContentDescriptors` with the `workshop.ContentDescriptor` enum.

```js
const item = await client.workshop.getItem(itemId, {
  includeMetadata: true,
  includeKeyValueTags: true,
  returnChildren: true,
  language: 'german',
})
```

All four item queries support these return options. Read `metadata`, `keyValueTags`, and `children` on each item. Omitted return options leave the corresponding result properties unset. Empty metadata is also unset. `requiredKeyValueTags` filters by exact key/value pairs. Only `getUserItems` accepts `cloudFileNameFilter`.

The `language` query option selects text to read. It does not select a language for updates. `previewPath` updates the primary image and `previewUrl` reads it. The existing `includeAdditionalPreviews` flag requests extra media from Steam, but the pinned Rust binding exposes neither those results nor an update-language setter. Neither feature is advertised as callable.

These bindings adapt [JDeffner's Workshop changes](https://github.com/JDeffner/steamworks.js/commit/2b611c2b448ae93c641ee3dd30cf2523557f8988) and the collection query in [newbe36524's PR #204](https://github.com/ceifa/steamworks.js/pull/204). The compatibility limits above also apply to the localized updates in [skyatgit's fork](https://github.com/skyatgit/steamworks.js/commit/908f4147c0e7af152bfa8b873110e30d6b521963) and [JDeffner's follow-up](https://github.com/JDeffner/steamworks.js/commit/44d4b94e6410205139858d13823df3f9d30cceca).
