---
id: mem-persist-ownership-guard-20261025
created: 2026-10-25
---
# Persistence ownership guard: two weaknesses (mono-desk)

Found by an independent read-only review of `src/persist.rs` on
2026-10-25, while closing the four plan initiatives. Neither is
exploited today — the settings path already fails closed and the
autostart path only ever writes the app's own
`std::env::current_exe()` — but both are one refactor away from
becoming real, so they are recorded rather than left implicit.

1. **`is_managed` is a whole-file substring test.** It is
   `fs::read_to_string(path).map(|text| text.contains(marker))`
   (`src/persist.rs:403-407`), and both `write_guarded`
   (`:411-418`) and `remove_guarded` (`:432-442`) trust it. A
   *foreign* file that merely mentions the marker string anywhere —
   in a comment, a quoted value — is therefore treated as ours and
   may be overwritten or deleted. The settings path does not share
   this flaw because `save_to` goes through `parse`, which requires
   the marker as the **first line** (`:195`, `:337-358`); it is the
   autostart path (`.desktop`, LaunchAgent plist) that relies on the
   weaker check.
   → Guard when touching this: keep any new managed-file check
   first-line-anchored or key-anchored, never a bare `contains`
   over the whole file.

2. **`desktop_entry` does not escape newlines.** It escapes only
   `\` and `"` (`src/persist.rs:486`), so a newline in `exe` would
   terminate the `Exec="..."` value (`:493`) and inject arbitrary
   `.desktop` keys. `plist` escapes `&`, `<`, `>` (`:502-506`) and so
   cannot be closed out of its `<string>`. Both are `pub fn` taking a
   plain `&str`, so only the current caller
   (`autostart_contents`, fed by `current_exe()` at `:550`) keeps
   them safe.
   → Guard: escaping for a file format must cover the format's own
   terminators (newline for `key=value`, `<`/`&` for XML), not just
   the quote character that happens to wrap the value.

Also confirmed clean, so future work need not re-derive it: the
temp+rename write is same-directory and `fsync`ed before rename
(`:373-401`), every settings string value is escaped by a true
round-trip pair (`:119-154`), and `src/persist.rs:1-622` has no
`unwrap`/`expect`/`panic!`/slice indexing at all.
