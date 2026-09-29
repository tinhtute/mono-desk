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
    /// Dark theme enabled.
    pub dark_mode: bool,
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
}

impl Default for StoredSettings {
    fn default() -> Self {
        Self {
            dark_mode: false,
            language: "en".to_string(),
            font_size: 14.0,
            launch_at_login: false,
            notifications_enabled: true,
            username: "ninja".to_string(),
            display_name: "Ninja".to_string(),
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
    out.push_str(&format!("dark_mode={}\n", s.dark_mode));
    out.push_str(&format!("language={}\n", escape(&s.language)));
    out.push_str(&format!("font_size={}\n", s.font_size));
    out.push_str(&format!("launch_at_login={}\n", s.launch_at_login));
    out.push_str(&format!(
        "notifications_enabled={}\n",
        s.notifications_enabled
    ));
    out.push_str(&format!("username={}\n", escape(&s.username)));
    out.push_str(&format!("display_name={}\n", escape(&s.display_name)));
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
    for line in lines {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key.trim() {
            "dark_mode" => {
                if let Some(b) = parse_bool(value) {
                    s.dark_mode = b;
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

/// Whether the file at `path` carries our ownership `marker`.
fn is_managed(path: &Path, marker: &str) -> bool {
    fs::read_to_string(path)
        .map(|text| text.contains(marker))
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
    // freedesktop quoting: inside double quotes, `\` and `"` must be
    // backslash-escaped or the Exec line mis-parses.
    let exe = exe.replace('\\', "\\\\").replace('"', "\\\"");
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
                dark_mode: true,
                language: "zh-CN".into(),
                font_size: 13.5,
                launch_at_login: true,
                notifications_enabled: false,
                username: "tinhpt".into(),
                display_name: "Tinh Pham".into(),
            },
        ] {
            assert_eq!(parse(&serialize(&s)), Some(s));
        }
    }

    #[test]
    fn round_trip_hostile_strings() {
        let s = StoredSettings {
            dark_mode: true,
            language: "ja".into(),
            font_size: 8.0,
            launch_at_login: false,
            notifications_enabled: true,
            // backslash, newline, CR, `=`, leading/trailing spaces
            username: "a\\b\nc=d\re ".into(),
            display_name: "  日本語 🎉 =x\\ ".into(),
        };
        let text = serialize(&s);
        // Hostile bytes must not create extra lines.
        assert_eq!(text.lines().count(), 8);
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
                    dark_mode=maybe\n\
                    dark_mode=true\n\
                    font_size=banana\n\
                    font_size=NaN\n\
                    font_size=inf\n\
                    language=vi\n";
        let s = parse(text).unwrap();
        assert!(s.dark_mode); // last valid wins
        assert_eq!(s.language, "vi");
        assert_eq!(s.font_size, 14.0); // malformed + non-finite skipped → default
        assert!(!s.launch_at_login); // missing key → default
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
            dark_mode: true,
            language: "vi".into(),
            font_size: 20.0,
            ..StoredSettings::default()
        };
        assert!(save_to(&path, &s));
        assert_eq!(load_from(&path), s);

        // Second save overwrites a healthy managed file.
        let s2 = StoredSettings {
            dark_mode: false,
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

    #[test]
    fn is_managed_checks_marker() {
        let dir = tempdir("managed");
        let _g = Cleanup(dir.clone());
        let path = dir.join("f");
        assert!(!is_managed(&path, "MARK")); // absent
        fs::write(&path, "no marker here").unwrap();
        assert!(!is_managed(&path, "MARK"));
        fs::write(&path, "has MARK inside").unwrap();
        assert!(is_managed(&path, "MARK"));
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
        // Managed file: write replaces, remove deletes.
        fs::write(&path, "MARK old").unwrap();
        assert!(write_guarded(&path, "MARK body", "MARK"));
        assert_eq!(fs::read_to_string(&path).unwrap(), "MARK body");
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
}
