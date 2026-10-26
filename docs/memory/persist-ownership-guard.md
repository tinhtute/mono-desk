---
id: mem-persist-ownership-guard-20261025
created: 2026-10-25
---
# Persistence ownership guard: two weaknesses (mono-desk)

Found by an independent read-only review of `src/persist.rs` on
2026-10-25 while closing the four plan initiatives, then **fixed the
same day**. Both were latent — the settings path already failed closed
and the autostart path only ever writes the app's own
`std::env::current_exe()` — but each was one refactor away from doing
real damage. The fixes are below.

## 1. `is_managed` was a whole-file substring test — FIXED

It was `text.contains(marker)`, and `write_guarded` and
`remove_guarded` both trust it. A *foreign* file that merely
*mentioned* the marker — in a comment, or as a fragment of a longer
value — was therefore treated as ours and could be overwritten or
deleted.

It is now line-anchored: `text.lines().any(|line| line.trim() ==
marker)`. Both formats we write put the marker alone on its line
(indented in the plist, column-zero in the `.desktop` file), so every
file we really wrote still matches, while a bare mention no longer
does. Covered by `is_managed_checks_marker` and
`write_and_remove_guarded_honor_marker` in `src/persist.rs`, which
assert that `"has MARK inside"` and `"# do not edit — MARK was here"`
are **not** managed.

→ Guard for future work: keep any new managed-file check
first-line-anchored, line-anchored, or key-anchored — never a bare
`contains` over the whole file.

## 2. `desktop_entry` did not escape newlines — FIXED

It escaped only `\` and `"`, so a newline in `exe` would terminate the
`Exec="..."` value and let the rest of the string inject arbitrary
`.desktop` keys.

It now escapes `\`, `"`, `$`, and `` ` `` for freedesktop quoting, and
writes `\n`, `\r`, `\t` as their spec-defined escapes; any other
control character is dropped. Covered by
`desktop_entry_has_marker_and_exe` plus a standalone check run over
seven inputs (including `"/tmp/a\nHidden=true\nExec=/bin/sh"`) that
confirmed no raw newline can survive into the `Exec=` line.

→ Guard: escaping for a file format must cover the format's own
terminators (newline for `key=value`, `<`/`&` for XML), not just the
quote character that happens to wrap the value.

## Still true, so future work need not re-derive it

- `plist` escaping (`&`, `<`, `>`) cannot be closed out of its
  `<string>`, though raw C0 controls would still make launchd reject.
- The temp+rename write is same-directory and `fsync`ed before the
  rename; both failure paths clean up their temp file. A kill
  mid-write can still leave `.{name}.tmp-<pid>` behind (no sweep), and
  the directory is never fsynced.
- `escape`/`unescape` for settings values is an exact inverse, so no
  `key=value` injection is possible.
- `src/persist.rs` production code (before `mod tests`) has no
  `unwrap`/`expect`/`panic!`/slice indexing.
