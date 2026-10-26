//! Zero-dependency settings persistence and OS autostart management.
//!
//! Settings live in a `key=value` file whose first line is the ownership
//! marker `# managed-by: mono-desk`. Autostart entries (a Linux `.desktop`
//! file or a macOS LaunchAgent plist) carry their own markers. A file at a
//! target path that lacks the marker is foreign: it is never modified or
//! deleted, only reported via `eprintln!`. Every write goes through a temp
//! file plus rename in the same directory; I/O errors are logged and never
//! panic.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// First line of every settings file written by [`save`]; also the
/// ownership check used by [`parse`] and the foreign-file guard.
const SETTINGS_MARKER: &str = "# managed-by: mono-desk";

/// Ownership marker embedded in the Linux `.desktop` autostart entry.
#[cfg(target_os = "linux")]
const AUTOSTART_MARKER: &str = "X-Mono-Desk-Managed=true";

/// Ownership marker embedded in the macOS LaunchAgent plist.
#[cfg(target_os = "macos")]
const AUTOSTART_MARKER: &str = "<key>MonoDeskManaged</key>";

/// Persisted settings mirror of the app's `AppSettings`.
///
/// `Default` mirrors `AppSettings::default()` — a missing, corrupt, or
/// foreign settings file must yield usable values (notably `font_size`
/// 14.0, never 0.0 which would collapse `set_rem_size` to zero).
#[derive(Clone, Debug, PartialEq)]
pub struct StoredSettings {
    /// Accent color as `#RRGGBB`, or empty for the theme's own brand color.
    pub accent: String,
    /// Theme choice: `"light"`, `"dark"`, or `"system"` to follow the OS.
    /// Replaces the pre-2026-10 `dark_mode` boolean, which [`parse`] still
    /// accepts as a fallback.
    pub appearance: String,
    /// UI locale: `"en"`, `"vi"`, `"zh-CN"`, or `"ja"`.
    pub language: String,
    /// UI font size; finite, expected `8.0..=32.0`.
    pub font_size: f64,
    /// Whether an OS autostart entry should exist.
    pub launch_at_login: bool,
    /// Whether notifications are enabled.
    pub notifications_enabled: bool,
    /// Login name shown on the profile page.
    pub username: String,
    /// Display name shown on the profile page.
    pub display_name: String,
    /// Notification channel: email.
    pub channel_email: bool,
    /// Notification channel: desktop.
    pub channel_desktop: bool,
    /// Notification channel: digest.
    pub channel_digest: bool,
    /// Digest cadence: `"immediate"`, `"daily"`, or `"weekly"`.
    pub digest: String,
    /// IANA time zone name, or `"system"` to follow the OS.
    pub timezone: String,
}

impl Default for StoredSettings {
    fn default() -> Self {
        Self {
            accent: String::new(),
            appearance: "light".to_string(),
            language: "en".to_string(),
            font_size: 14.0,
            launch_at_login: false,
            notifications_enabled: true,
            username: "ninja".to_string(),
            display_name: "Ninja".to_string(),
            channel_email: true,
            channel_desktop: true,
            channel_digest: false,
            digest: "immediate".to_string(),
            timezone: "system".to_string(),
        }
    }
}

/// `$HOME`, or `None` when unset or empty.
fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .filter(|h| !h.is_empty())
        .map(PathBuf::from)
}

/// Platform config dir: `$XDG_CONFIG_HOME/mono-desk`, else
/// `$HOME/.config/mono-desk` (macOS: `$HOME/Library/Application
/// Support/mono-desk`). `None` if no home.
pub fn config_dir() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        home_dir().map(|h| {
            h.join("Library")
                .join("Application Support")
                .join("mono-desk")
        })
    }
    #[cfg(not(target_os = "macos"))]
    {
        if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME").filter(|v| !v.is_empty()) {
            return Some(PathBuf::from(xdg).join("mono-desk"));
        }
        home_dir().map(|h| h.join(".config").join("mono-desk"))
    }
}

/// Path of the settings file: `config_dir()/settings.conf`.
pub fn settings_path() -> Option<PathBuf> {
    config_dir().map(|d| d.join("settings.conf"))
}

/// Escape a value for one `key=value` line: `\`, newline, and carriage
/// return cannot appear literally.
fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            _ => out.push(c),
        }
    }
    out
}

/// Inverse of [`escape`]. Unknown escape sequences are kept verbatim so
/// foreign input loses nothing.
fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('\\') => out.push('\\'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Serialize with first line `# managed-by: mono-desk`, then `key=value`
/// lines; `\` and newline are escaped in values.
pub fn serialize(s: &StoredSettings) -> String {
    let mut out = String::new();
    out.push_str(SETTINGS_MARKER);
    out.push('\n');
    out.push_str(&format!("accent={}\n", escape(&s.accent)));
    out.push_str(&format!("appearance={}\n", escape(&s.appearance)));
    out.push_str(&format!("language={}\n", escape(&s.language)));
    out.push_str(&format!("font_size={}\n", s.font_size));
    out.push_str(&format!("launch_at_login={}\n", s.launch_at_login));
    out.push_str(&format!(
        "notifications_enabled={}\n",
        s.notifications_enabled
    ));
    out.push_str(&format!("username={}\n", escape(&s.username)));
    out.push_str(&format!("display_name={}\n", escape(&s.display_name)));
    out.push_str(&format!("channel_email={}\n", s.channel_email));
    out.push_str(&format!("channel_desktop={}\n", s.channel_desktop));
    out.push_str(&format!("channel_digest={}\n", s.channel_digest));
    out.push_str(&format!("digest={}\n", escape(&s.digest)));
    out.push_str(&format!("timezone={}\n", escape(&s.timezone)));
    out
}

/// `"true"`/`"false"` only; anything else is malformed.
fn parse_bool(value: &str) -> Option<bool> {
    match value.trim() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

/// Parse a settings file. `None` when the ownership marker is missing
/// (foreign file). Unknown keys are ignored, missing keys take defaults,
/// malformed lines are skipped; never panics.
pub fn parse(text: &str) -> Option<StoredSettings> {
    let mut lines = text.lines();
    if lines.next().map(str::trim_end) != Some(SETTINGS_MARKER) {
        return None;
    }
    let mut s = StoredSettings::default();
    // Set once an explicit `appearance` line lands, so a legacy `dark_mode`
    // line later in the file cannot override a newer choice.
    let mut saw_appearance = false;
    for line in lines {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key.trim() {
            // Only known appearances stick; anything else falls back to the
            // default rather than surfacing as an unselectable value.
            "appearance" => {
                if let known @ ("light" | "dark" | "system") = unescape(value).as_str() {
                    s.appearance = known.to_string();
                    saw_appearance = true;
                }
            }
            // Written before 2026-10: still honoured when the file carries no
            // `appearance` line, so an existing config keeps its mode.
            "dark_mode" => {
                if !saw_appearance && let Some(b) = parse_bool(value) {
                    s.appearance = if b { "dark" } else { "light" }.to_string();
                }
            }
            // Any string is kept; the app validates it and falls back to the
            // theme's own color. Empty means "no override".
            "accent" => {
                let accent = unescape(value);
                if !accent.is_empty() {
                    s.accent = accent;
                }
            }
            "language" => s.language = unescape(value),
            "font_size" => {
                if let Ok(f) = value.trim().parse::<f64>()
                    && f.is_finite()
                {
                    // Same bound as the NumberField control (8–32): a
                    // hand-edited file cannot push rem size out of range.
                    s.font_size = f.clamp(8.0, 32.0);
                }
            }
            "launch_at_login" => {
                if let Some(b) = parse_bool(value) {
                    s.launch_at_login = b;
                }
            }
            "notifications_enabled" => {
                if let Some(b) = parse_bool(value) {
                    s.notifications_enabled = b;
                }
            }
            "username" => s.username = unescape(value),
            "display_name" => s.display_name = unescape(value),
            "channel_email" => {
                if let Some(b) = parse_bool(value) {
                    s.channel_email = b;
                }
            }
            "channel_desktop" => {
                if let Some(b) = parse_bool(value) {
                    s.channel_desktop = b;
                }
            }
            "channel_digest" => {
                if let Some(b) = parse_bool(value) {
                    s.channel_digest = b;
                }
            }
            // Only known cadences stick; anything else falls back to the
            // default rather than surfacing as an unselectable value.
            "digest" => {
                if let known @ ("immediate" | "daily" | "weekly") = unescape(value).as_str() {
                    s.digest = known.to_string();
                }
            }
            "timezone" => {
                let tz = unescape(value);
                if !tz.is_empty() {
                    s.timezone = tz;
                }
            }
            _ => {} // unknown key: ignore
        }
    }
    Some(s)
}

/// Load the settings file; defaults when absent, corrupt, or foreign
/// (logs on corrupt or foreign). Never panics.
pub fn load() -> StoredSettings {
    match settings_path() {
        Some(path) => load_from(&path),
        None => StoredSettings::default(),
    }
}

/// [`load`] against an explicit path, for tests.
fn load_from(path: &Path) -> StoredSettings {
    match fs::read_to_string(path) {
        Ok(text) => match parse(&text) {
            Some(s) => s,
            None => {
                eprintln!(
                    "persist: {} is foreign or corrupt (missing marker); using defaults",
                    path.display()
                );
                StoredSettings::default()
            }
        },
        Err(e) if e.kind() == io::ErrorKind::NotFound => StoredSettings::default(),
        Err(e) => {
            eprintln!(
                "persist: cannot read {}: {e}; using defaults",
                path.display()
            );
            StoredSettings::default()
        }
    }
}

/// Save atomically (temp file + rename in the same directory). Refuses to
/// overwrite a foreign file (exists but no marker): logs and returns
/// `false`. Other errors log and return `false`.
pub fn save(s: &StoredSettings) -> bool {
    match settings_path() {
        Some(path) => save_to(&path, s),
        None => {
            eprintln!("persist: no config directory (missing home); cannot save");
            false
        }
    }
}

/// [`save`] against an explicit path, for tests.
fn save_to(path: &Path, s: &StoredSettings) -> bool {
    if path.exists() {
        match fs::read_to_string(path) {
            // A managed file (marker present, however corrupt) may be
            // healed by overwriting; a markerless file is foreign.
            Ok(text) => {
                if parse(&text).is_none() {
                    eprintln!(
                        "persist: refusing to overwrite foreign file {}",
                        path.display()
                    );
                    return false;
                }
            }
            Err(e) => {
                eprintln!(
                    "persist: cannot read {}: {e}; refusing to overwrite",
                    path.display()
                );
                return false;
            }
        }
    }
    let Some(dir) = path.parent() else {
        eprintln!("persist: {} has no parent directory", path.display());
        return false;
    };
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        eprintln!("persist: {} has no usable file name", path.display());
        return false;
    };
    atomic_write(dir, name, &serialize(s))
}

/// Write `contents` to `dir/name` via a temp file + rename. Creates `dir`
/// when missing. Logs and returns `false` on any error; never leaves the
/// temp file behind.
fn atomic_write(dir: &Path, name: &str, contents: &str) -> bool {
    if let Err(e) = fs::create_dir_all(dir) {
        eprintln!("persist: cannot create {}: {e}", dir.display());
        return false;
    }
    let tmp = dir.join(format!(".{name}.tmp-{}", std::process::id()));
    let target = dir.join(name);
    let written = (|| -> io::Result<()> {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(contents.as_bytes())?;
        f.sync_all()
    })();
    if let Err(e) = written {
        eprintln!("persist: cannot write {}: {e}", tmp.display());
        let _ = fs::remove_file(&tmp);
        return false;
    }
    if let Err(e) = fs::rename(&tmp, &target) {
        eprintln!(
            "persist: cannot rename {} -> {}: {e}",
            tmp.display(),
            target.display()
        );
        let _ = fs::remove_file(&tmp);
        return false;
    }
    true
}

/// Whether the file at `path` carries our ownership `marker` on a line of
/// its own.
///
/// The check is line-anchored on purpose: a whole-file `contains` would
/// treat any foreign file that merely *mentions* the marker — inside a
/// comment, or as a fragment of a longer value — as ours, and
/// `write_guarded`/`remove_guarded` would then overwrite or delete it.
/// Both formats we write put the marker alone on its line (indented in the
/// plist, column-zero in the `.desktop` file), so comparing trimmed lines
/// accepts every file we really wrote and rejects one that only quotes the
/// marker.
fn is_managed(path: &Path, marker: &str) -> bool {
    fs::read_to_string(path)
        .map(|text| text.lines().any(|line| line.trim() == marker))
        .unwrap_or(false)
}

/// Atomically write `contents` to `path`, refusing when a foreign file
/// (exists, no marker) occupies it.
fn write_guarded(path: &Path, contents: &str, marker: &str) -> bool {
    if path.exists() && !is_managed(path, marker) {
        eprintln!(
            "persist: refusing to overwrite foreign file {}",
            path.display()
        );
        return false;
    }
    let Some(dir) = path.parent() else {
        eprintln!("persist: {} has no parent directory", path.display());
        return false;
    };
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        eprintln!("persist: {} has no usable file name", path.display());
        return false;
    };
    atomic_write(dir, name, contents)
}

/// Remove `path` only when it carries `marker`. Absent is already
/// converged (`true`); foreign is refused (`false`).
fn remove_guarded(path: &Path, marker: &str) -> bool {
    if !path.exists() {
        return true;
    }
    if !is_managed(path, marker) {
        eprintln!(
            "persist: refusing to remove foreign file {}",
            path.display()
        );
        return false;
    }
    match fs::remove_file(path) {
        Ok(()) => true,
        Err(e) => {
            eprintln!("persist: cannot remove {}: {e}", path.display());
            false
        }
    }
}

/// Autostart entry path: Linux `$XDG_CONFIG_HOME/autostart/
/// mono-desk.desktop` (or `~/.config/autostart/…`), macOS
/// `~/Library/LaunchAgents/com.mono-desk.plist`. `None` on other
/// platforms or when there is no home.
pub fn autostart_path() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .or_else(|| home_dir().map(|h| h.join(".config")))?;
        Some(base.join("autostart").join("mono-desk.desktop"))
    }
    #[cfg(target_os = "macos")]
    {
        home_dir().map(|h| {
            h.join("Library")
                .join("LaunchAgents")
                .join("com.mono-desk.plist")
        })
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        None
    }
}

/// Linux `.desktop` entry contents, carrying the
/// `X-Mono-Desk-Managed=true` ownership marker. `exe` is the
/// `std::env::current_exe()` display path.
#[cfg(target_os = "linux")]
pub fn desktop_entry(exe: &str) -> String {
    // freedesktop quoting: inside double quotes `\`, `"`, `$` and a
    // backtick must be backslash-escaped or the Exec line mis-parses.
    let mut escaped = String::with_capacity(exe.len());
    for ch in exe.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '$' => escaped.push_str("\\$"),
            '`' => escaped.push_str("\\`"),
            // A raw newline would end the `Exec=` line and let the rest of
            // the string inject arbitrary `.desktop` keys. Control
            // characters are written as their `\n`/`\t`/`\r` escapes,
            // which the spec defines for string values; anything else
            // unprintable is dropped.
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            c if c.is_control() => {}
            c => escaped.push(c),
        }
    }
    let exe = escaped;
    format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Version=1.0\n\
         Name=mono-desk\n\
         Comment=mono-desk\n\
         Exec=\"{exe}\"\n\
         Terminal=false\n\
         X-GNOME-Autostart-enabled=true\n\
         {AUTOSTART_MARKER}\n"
    )
}

/// Minimal XML escaping for plist string values.
#[cfg(target_os = "macos")]
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// macOS LaunchAgent plist contents, carrying the
/// `<key>MonoDeskManaged</key><true/>` ownership marker. `exe` is the
/// `std::env::current_exe()` display path.
#[cfg(target_os = "macos")]
pub fn plist(exe: &str) -> String {
    let exe = xml_escape(exe);
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \
         \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
         <plist version=\"1.0\">\n\
         <dict>\n\
         \t<key>Label</key>\n\
         \t<string>com.mono-desk</string>\n\
         \t<key>ProgramArguments</key>\n\
         \t<array>\n\
         \t\t<string>{exe}</string>\n\
         \t</array>\n\
         \t<key>RunAtLoad</key>\n\
         \t<true/>\n\
         \t<key>MonoDeskManaged</key>\n\
         \t<true/>\n\
         </dict>\n\
         </plist>\n"
    )
}

/// Entry body for the current platform's autostart format.
#[cfg(target_os = "linux")]
fn autostart_contents(exe: &str) -> String {
    desktop_entry(exe)
}

/// Entry body for the current platform's autostart format.
#[cfg(target_os = "macos")]
fn autostart_contents(exe: &str) -> String {
    plist(exe)
}

/// Install the autostart entry at an explicit path, for tests.
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn install_at(path: &Path) -> bool {
    match std::env::current_exe() {
        Ok(exe) => write_guarded(
            path,
            &autostart_contents(&exe.display().to_string()),
            AUTOSTART_MARKER,
        ),
        Err(e) => {
            eprintln!("persist: cannot resolve current_exe: {e}");
            false
        }
    }
}

/// Remove the autostart entry at an explicit path, for tests.
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn remove_at(path: &Path) -> bool {
    remove_guarded(path, AUTOSTART_MARKER)
}

/// Install the autostart entry for `current_exe`; guarded: a foreign
/// file at the target path is logged and left (`false`). Errors log and
/// return `false`. No-op (`true`) on unsupported platforms.
#[cfg(any(target_os = "linux", target_os = "macos"))]
pub fn autostart_install() -> bool {
    match autostart_path() {
        Some(path) => install_at(&path),
        None => {
            eprintln!("persist: no autostart path (missing home); cannot install");
            false
        }
    }
}

/// No-op on unsupported platforms.
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub fn autostart_install() -> bool {
    true
}

/// Remove the autostart entry only if it exists and carries the marker;
/// foreign file → log + `false`; absent → `true` (already converged).
/// No-op (`true`) on unsupported platforms.
#[cfg(any(target_os = "linux", target_os = "macos"))]
pub fn autostart_remove() -> bool {
    match autostart_path() {
        Some(path) => remove_at(&path),
        // Nowhere for an entry to exist: already converged.
        None => true,
    }
}

/// No-op on unsupported platforms.
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub fn autostart_remove() -> bool {
    true
}

/// Converge the autostart entry to `enabled`: install when `true`,
/// remove when `false`.
#[cfg(any(target_os = "linux", target_os = "macos"))]
pub fn autostart_converge(enabled: bool) -> bool {
    if enabled {
        autostart_install()
    } else {
        autostart_remove()
    }
}

/// No-op on unsupported platforms.
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub fn autostart_converge(_enabled: bool) -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    /// Unique temp subdirectory per test.
    fn tempdir(tag: &str) -> PathBuf {
        static N: AtomicU64 = AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "mono-desk-persist-test-{}-{tag}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Removes the temp dir when the test ends.
    struct Cleanup(PathBuf);

    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn serialize_starts_with_marker() {
        let text = serialize(&StoredSettings::default());
        assert_eq!(text.lines().next(), Some(SETTINGS_MARKER));
    }

    #[test]
    fn round_trip_default_and_populated() {
        for s in [
            StoredSettings::default(),
            StoredSettings {
                accent: "#2e6fd0".into(),
                appearance: "system".into(),
                language: "zh-CN".into(),
                font_size: 13.5,
                launch_at_login: true,
                notifications_enabled: false,
                username: "tinhpt".into(),
                display_name: "Tinh Pham".into(),
                channel_email: false,
                channel_desktop: true,
                channel_digest: true,
                digest: "weekly".into(),
                timezone: "Asia/Tokyo".into(),
            },
        ] {
            assert_eq!(parse(&serialize(&s)), Some(s));
        }
    }

    #[test]
    fn round_trip_hostile_strings() {
        let s = StoredSettings {
            accent: "#bb1b1b".into(),
            appearance: "dark".into(),
            language: "ja".into(),
            font_size: 8.0,
            launch_at_login: false,
            notifications_enabled: true,
            // backslash, newline, CR, `=`, leading/trailing spaces
            username: "a\\b\nc=d\re ".into(),
            display_name: "  日本語 🎉 =x\\ ".into(),
            channel_email: true,
            channel_desktop: false,
            channel_digest: true,
            digest: "daily".into(),
            timezone: "Europe/London".into(),
        };
        let text = serialize(&s);
        // Hostile bytes must not create extra lines.
        assert_eq!(text.lines().count(), 14);
        assert_eq!(parse(&text), Some(s));
        // Empty strings round-trip too.
        let empty = StoredSettings::default();
        assert_eq!(parse(&serialize(&empty)), Some(empty));
    }

    #[test]
    fn parse_requires_marker() {
        assert_eq!(parse(""), None);
        assert_eq!(parse("dark_mode=true\n"), None);
        assert_eq!(parse("# managed-by: other\ndark_mode=true"), None);
        // Marker not on the first line is still foreign.
        assert_eq!(parse("dark_mode=true\n# managed-by: mono-desk\n"), None);
    }

    #[test]
    fn parse_ignores_unknown_and_malformed() {
        let text = "# managed-by: mono-desk\n\
                    # a comment\n\
                    \n\
                    unknown_key=1\n\
                    no-equals-line\n\
                    appearance=maybe\n\
                    appearance=system\n\
                    font_size=banana\n\
                    font_size=NaN\n\
                    font_size=inf\n\
                    digest=bogus\n\
                    timezone=\n\
                    timezone=Asia/Tokyo\n\
                    language=vi\n";
        let s = parse(text).unwrap();
        assert_eq!(s.appearance, "system"); // unknown word skipped, last valid wins
        assert_eq!(s.language, "vi");
        assert_eq!(s.font_size, 14.0); // malformed + non-finite skipped → default
        assert!(!s.launch_at_login); // missing key → default
        assert_eq!(s.digest, "immediate"); // unknown cadence → default
        assert_eq!(s.timezone, "Asia/Tokyo"); // empty skipped, valid wins
    }

    /// A config written before 2026-10 carries `dark_mode` instead of
    /// `appearance`; it must still select the mode the owner chose, and an
    /// explicit `appearance` must win regardless of line order.
    #[test]
    fn legacy_dark_mode_still_selects_an_appearance() {
        let parsed = |body: &str| {
            parse(&format!("{SETTINGS_MARKER}\n{body}"))
                .unwrap()
                .appearance
        };

        assert_eq!(parsed("dark_mode=true\n"), "dark");
        assert_eq!(parsed("dark_mode=false\n"), "light");
        assert_eq!(parsed("dark_mode=maybe\n"), "light"); // malformed → default

        // An explicit appearance wins, whichever line comes first.
        assert_eq!(parsed("dark_mode=true\nappearance=system\n"), "system");
        assert_eq!(parsed("appearance=system\ndark_mode=true\n"), "system");
        // An unknown appearance word leaves the legacy key in charge.
        assert_eq!(parsed("appearance=bogus\ndark_mode=true\n"), "dark");
    }

    #[test]
    fn load_absent_corrupt_and_foreign_yield_defaults() {
        let dir = tempdir("load");
        let _g = Cleanup(dir.clone());
        let path = dir.join("settings.conf");

        // Absent → defaults.
        assert_eq!(load_from(&path), StoredSettings::default());

        // Foreign (no marker) → defaults.
        fs::write(&path, "somebody else's file\n").unwrap();
        assert_eq!(load_from(&path), StoredSettings::default());

        // Corrupt (marker, garbage body) → defaults for bad lines.
        fs::write(&path, "# managed-by: mono-desk\n@@@\nfont_size=18\n").unwrap();
        let s = load_from(&path);
        assert_eq!(s.font_size, 18.0);
        assert_eq!(s.language, "en");
    }

    #[test]
    fn save_refuses_foreign_file() {
        let dir = tempdir("foreign");
        let _g = Cleanup(dir.clone());
        let path = dir.join("settings.conf");
        fs::write(&path, "somebody else's file\n").unwrap();

        assert!(!save_to(&path, &StoredSettings::default()));
        // Untouched.
        assert_eq!(fs::read_to_string(&path).unwrap(), "somebody else's file\n");
    }

    #[test]
    fn save_overwrites_managed_and_heals_corrupt() {
        let dir = tempdir("save");
        let _g = Cleanup(dir.clone());
        let path = dir.join("settings.conf");

        // Corrupt-but-managed file is healed by save.
        fs::write(&path, "# managed-by: mono-desk\n@@@garbage").unwrap();
        let s = StoredSettings {
            appearance: "dark".into(),
            language: "vi".into(),
            font_size: 20.0,
            ..StoredSettings::default()
        };
        assert!(save_to(&path, &s));
        assert_eq!(load_from(&path), s);

        // Second save overwrites a healthy managed file.
        let s2 = StoredSettings {
            appearance: "system".into(),
            ..s.clone()
        };
        assert!(save_to(&path, &s2));
        assert_eq!(load_from(&path), s2);
    }

    #[test]
    fn atomic_write_creates_dirs_and_leaves_no_temp() {
        let dir = tempdir("atomic");
        let _g = Cleanup(dir.clone());
        let nested = dir.join("a").join("b");
        assert!(atomic_write(&nested, "x.conf", "data"));
        assert_eq!(fs::read_to_string(nested.join("x.conf")).unwrap(), "data");
        // Exactly one entry: the target; no temp file left behind.
        let entries: Vec<_> = fs::read_dir(&nested).unwrap().collect();
        assert_eq!(entries.len(), 1);
    }

    /// The ownership check is line-anchored: a foreign file that merely
    /// *mentions* the marker must not be treated as ours, or a guarded
    /// write/remove would clobber it. Both formats we write put the marker
    /// alone on its line (indented in the plist, column-zero in the
    /// `.desktop` file).
    #[test]
    fn is_managed_checks_marker() {
        let dir = tempdir("managed");
        let _g = Cleanup(dir.clone());
        let path = dir.join("f");
        assert!(!is_managed(&path, "MARK")); // absent
        fs::write(&path, "no marker here").unwrap();
        assert!(!is_managed(&path, "MARK"));
        // Marker alone on a line — our own shape — is ours.
        fs::write(&path, "body\nMARK\n").unwrap();
        assert!(is_managed(&path, "MARK"));
        // Indented marker is ours too (the plist's `<key>` line).
        fs::write(&path, "<dict>\n\tMARK\n</dict>").unwrap();
        assert!(is_managed(&path, "MARK"));
        // A bare mention inside a longer line is NOT ours: that is exactly
        // the case that let `is_managed` clobber foreign files.
        fs::write(&path, "has MARK inside").unwrap();
        assert!(!is_managed(&path, "MARK"));
        // A comment that quotes the marker is NOT ours either.
        fs::write(&path, "# do not edit — MARK was here\n").unwrap();
        assert!(!is_managed(&path, "MARK"));
    }

    #[test]
    fn write_and_remove_guarded_honor_marker() {
        let dir = tempdir("guarded");
        let _g = Cleanup(dir.clone());
        let path = dir.join("entry");

        // Foreign file: write and remove both refuse, file untouched.
        fs::write(&path, "foreign").unwrap();
        assert!(!write_guarded(&path, "MARK body", "MARK"));
        assert_eq!(fs::read_to_string(&path).unwrap(), "foreign");
        assert!(!remove_guarded(&path, "MARK"));
        assert_eq!(fs::read_to_string(&path).unwrap(), "foreign");
        // Foreign file that only *mentions* the marker still refuses.
        fs::write(&path, "foreign with MARK quoted").unwrap();
        assert!(!write_guarded(&path, "MARK body", "MARK"));
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "foreign with MARK quoted"
        );
        // Managed file: marker alone on its line. Write replaces, remove
        // deletes.
        fs::write(&path, "[Desktop Entry]\nMARK\n").unwrap();
        assert!(write_guarded(
            &path,
            "[Desktop Entry]\nMARK\nbody\n",
            "MARK"
        ));
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "[Desktop Entry]\nMARK\nbody\n"
        );
        assert!(remove_guarded(&path, "MARK"));
        assert!(!path.exists());
        // Absent remove is already converged.
        assert!(remove_guarded(&path, "MARK"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn desktop_entry_has_marker_and_exe() {
        let e = desktop_entry("/opt/mono desk/mono-desk");
        assert!(e.starts_with("[Desktop Entry]"));
        assert!(e.contains(AUTOSTART_MARKER));
        assert!(e.contains("Exec=\"/opt/mono desk/mono-desk\""));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn plist_has_marker_and_exe() {
        let p = plist("/Applications/Mono Desk.app/Contents/MacOS/mono-desk");
        assert!(p.contains(AUTOSTART_MARKER));
        assert!(
            p.contains("<string>/Applications/Mono Desk.app/Contents/MacOS/mono-desk</string>")
        );
        assert!(p.contains("<key>RunAtLoad</key>"));
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    #[test]
    fn autostart_install_remove_honor_marker() {
        let dir = tempdir("autostart");
        let _g = Cleanup(dir.clone());
        let path = dir.join("entry");

        // Install creates a managed entry naming current_exe.
        assert!(install_at(&path));
        let body = fs::read_to_string(&path).unwrap();
        assert!(body.contains(AUTOSTART_MARKER));
        let exe = std::env::current_exe().unwrap().display().to_string();
        assert!(body.contains(&exe));

        // Re-install over a managed entry is fine.
        assert!(install_at(&path));

        // Remove deletes the managed entry; absent remove converges.
        assert!(remove_at(&path));
        assert!(!path.exists());
        assert!(remove_at(&path));

        // Foreign file: install and remove both refuse, file untouched.
        fs::write(&path, "foreign entry").unwrap();
        assert!(!install_at(&path));
        assert_eq!(fs::read_to_string(&path).unwrap(), "foreign entry");
        assert!(!remove_at(&path));
        assert!(path.exists());
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    #[test]
    fn converge_at_installs_and_removes() {
        let dir = tempdir("converge");
        let _g = Cleanup(dir.clone());
        let path = dir.join("entry");

        assert!(install_at(&path));
        assert!(path.exists());
        assert!(remove_at(&path));
        assert!(!path.exists());
        // Remove on absent is already converged.
        assert!(remove_at(&path));
        // Remove on foreign refuses and leaves it.
        fs::write(&path, "foreign").unwrap();
        assert!(!remove_at(&path));
        assert!(path.exists());
    }

    /// The ownership check is line-anchored: a foreign file that merely
    /// *mentions* the marker must not be treated as ours, or a guarded
    /// write/remove would clobber it.
    #[test]
    fn is_managed_requires_the_marker_on_its_own_line() {
        let dir = tempdir("managed");
        let _g = Cleanup(dir.clone());
        let path = dir.join("entry");
        let marker = "X-Mono-Desk-Managed=true";

        // Our own shape: marker alone on a line, trimmed either way.
        fs::write(
            &path,
            "[Desktop Entry]\nExec=\"/usr/bin/mono\"\nX-Mono-Desk-Managed=true\n",
        )
        .unwrap();
        assert!(is_managed(&path, marker));

        // Indented (the plist's `<key>` line has leading whitespace) is ours.
        fs::write(&path, "<dict>\n\t<key>MonoDeskManaged</key>\n</dict>").unwrap();
        assert!(is_managed(&path, "<key>MonoDeskManaged</key>"));

        // A comment that mentions the marker is NOT ours.
        fs::write(&path, "# do not edit — X-Mono-Desk-Managed=true was here\n").unwrap();
        assert!(!is_managed(&path, marker));

        // A value that merely contains the marker as a substring is NOT ours.
        fs::write(&path, "Comment=X-Mono-Desk-Managed=true and more\n").unwrap();
        assert!(!is_managed(&path, marker));

        // Absent path is not managed.
        assert!(!is_managed(&dir.join("missing"), marker));
    }
}
