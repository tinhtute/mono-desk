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

// Settings-domain defaults/clamps: the pre-port `src/theme.rs` owned them; on
// the zeron stack they are app-owned (see `crate::app_settings`), since the
// vendored `zeron_ui::theme` owns the paint tokens instead.
use crate::app_settings::{
    CODE_FONT_SIZE_DEFAULT, FOCAL_DEFAULT, MATH_SIZE_DEFAULT, MATH_SIZE_MAX, MATH_SIZE_MIN,
    MathFont, NewThreadBackgroundEffect, SIDEBAR_TONE_DEFAULT, SIDEBAR_WIDTH_DEFAULT,
    TERMINAL_FONT_SIZE_DEFAULT, THEME_DARK_DEFAULT, THEME_LIGHT_DEFAULT, TRANSCRIPT_WIDTH_DEFAULT,
    WINDOW_OPACITY_DEFAULT, ZOOM_DEFAULT, clamp_focal, clamp_sidebar_tone, clamp_sidebar_width,
    clamp_ui_font_size, clamp_window_opacity, clamp_zoom, font_family_from_word, font_family_word,
    normalize_transcript_width, reduce_motion_from_word, reduce_motion_word,
    wallpaper_color_from_str, wallpaper_color_to_str,
};

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
    /// Whether the window background is translucent (the glass master
    /// switch). `false` renders the opaque palette.
    pub background_translucent: bool,
    /// The single frost-shell opacity in percent while translucent; finite,
    /// expected `40.0..=100.0` (opaque).
    pub window_opacity: f64,
    /// Sidebar ink tone in percent of the max wash, `0.0..=100.0`; painted
    /// over the frost, never a second surface.
    pub sidebar_tone: f64,
    /// Persisted rail width in logical pixels, `242.0..=400.0`.
    pub sidebar_width: f64,
    /// Whether the sidebar is collapsed to zero width (zeron's
    /// `UiSettings.sidebar_collapsed`). Additive key: files written before it
    /// existed read as `false` (open), and older builds ignore it.
    pub sidebar_collapsed: bool,
    /// Whether the window asks the platform for a blurred (frosted)
    /// backdrop. Only meaningful while `background_translucent` is on.
    pub background_blur: bool,
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
    /// Registry variant id used in light appearance.
    pub theme_light: String,
    /// Registry variant id used in dark appearance.
    pub theme_dark: String,
    /// Interface font-family word (`momo-trust-sans`, `installed:<name>`, …).
    pub ui_font_family: String,
    /// Terminal font-family word.
    pub terminal_font_family: String,
    /// Code/diff font-family word.
    pub code_font_family: String,
    /// Terminal font size in px, `8.0..=32.0`.
    pub terminal_font_size: f64,
    /// Code/diff font size in px, `8.0..=32.0`.
    pub code_font_size: f64,
    /// Transcript column width in px, `560.0..=1200.0` on the 16 px grid.
    pub transcript_width: f64,
    /// Whether Markdown math is rendered.
    pub render_math: bool,
    /// Math face key (`fira`, `stixTwo`, …).
    pub math_font: String,
    /// Math size as a percent of the surrounding text, `90..=150`.
    pub math_size: u16,
    /// Whether mouse-wheel notches glide instead of jumping.
    pub smooth_scrolling: bool,
    /// Reduced-motion word (`system`, `on`, `off`).
    pub reduce_motion: String,
    /// Also snap animations while the main window is not focused.
    pub pause_in_background: bool,
    /// Whether the wallpaper tints the theme's accent.
    pub wallpaper_theme_colors: bool,
    /// Sampled wallpaper color as `#RRGGBB`, or empty.
    pub wallpaper_color: String,
    /// Device-local wallpaper folder, or empty.
    pub wallpaper_folder: String,
    /// Most recently displayed wallpaper source, or empty.
    pub wallpaper_source: String,
    /// Managed new-thread background image path, or empty.
    pub new_thread_background_path: String,
    /// Original file name of that image, shown in the row.
    pub new_thread_background_name: String,
    /// Normalized horizontal framing of the image, `0.0..=1.0`.
    pub new_thread_background_focal_x: f64,
    /// Normalized vertical framing of the image, `0.0..=1.0`.
    pub new_thread_background_focal_y: f64,
    /// Image zoom, `1.0..=4.0`.
    pub new_thread_background_zoom: f64,
    /// Background treatment word (`none`, `dither`, `ascii`, …).
    pub new_thread_background_effect: String,
}

impl Default for StoredSettings {
    fn default() -> Self {
        Self {
            accent: String::new(),
            appearance: "light".to_string(),
            language: "en".to_string(),
            font_size: 14.0,
            background_translucent: false,
            window_opacity: WINDOW_OPACITY_DEFAULT,
            sidebar_tone: SIDEBAR_TONE_DEFAULT,
            sidebar_width: SIDEBAR_WIDTH_DEFAULT,
            sidebar_collapsed: false,
            background_blur: false,
            launch_at_login: false,
            notifications_enabled: true,
            username: "ninja".to_string(),
            display_name: "Ninja".to_string(),
            channel_email: true,
            channel_desktop: true,
            channel_digest: false,
            digest: "immediate".to_string(),
            timezone: "system".to_string(),
            theme_light: THEME_LIGHT_DEFAULT.to_string(),
            theme_dark: THEME_DARK_DEFAULT.to_string(),
            ui_font_family: "momo-trust-sans".to_string(),
            terminal_font_family: "xanh-mono".to_string(),
            code_font_family: "google-sans-code".to_string(),
            terminal_font_size: TERMINAL_FONT_SIZE_DEFAULT,
            code_font_size: CODE_FONT_SIZE_DEFAULT,
            transcript_width: TRANSCRIPT_WIDTH_DEFAULT,
            render_math: true,
            math_font: "fira".to_string(),
            math_size: MATH_SIZE_DEFAULT,
            smooth_scrolling: true,
            reduce_motion: "system".to_string(),
            pause_in_background: false,
            wallpaper_theme_colors: false,
            wallpaper_color: String::new(),
            wallpaper_folder: String::new(),
            wallpaper_source: String::new(),
            new_thread_background_path: String::new(),
            new_thread_background_name: String::new(),
            new_thread_background_focal_x: f64::from(FOCAL_DEFAULT),
            new_thread_background_focal_y: f64::from(FOCAL_DEFAULT),
            new_thread_background_zoom: f64::from(ZOOM_DEFAULT),
            new_thread_background_effect: "none".to_string(),
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
    out.push_str(&format!(
        "background_translucent={}\n",
        s.background_translucent
    ));
    out.push_str(&format!("window_opacity={}\n", s.window_opacity));
    out.push_str(&format!("sidebar_tone={}\n", s.sidebar_tone));
    out.push_str(&format!("sidebar_width={}\n", s.sidebar_width));
    out.push_str(&format!("sidebar_collapsed={}\n", s.sidebar_collapsed));
    out.push_str(&format!("background_blur={}\n", s.background_blur));
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
    out.push_str(&format!("theme_light={}\n", escape(&s.theme_light)));
    out.push_str(&format!("theme_dark={}\n", escape(&s.theme_dark)));
    out.push_str(&format!("ui_font_family={}\n", escape(&s.ui_font_family)));
    out.push_str(&format!(
        "terminal_font_family={}\n",
        escape(&s.terminal_font_family)
    ));
    out.push_str(&format!(
        "code_font_family={}\n",
        escape(&s.code_font_family)
    ));
    out.push_str(&format!("terminal_font_size={}\n", s.terminal_font_size));
    out.push_str(&format!("code_font_size={}\n", s.code_font_size));
    out.push_str(&format!("transcript_width={}\n", s.transcript_width));
    out.push_str(&format!("render_math={}\n", s.render_math));
    out.push_str(&format!("math_font={}\n", escape(&s.math_font)));
    out.push_str(&format!("math_size={}\n", s.math_size));
    out.push_str(&format!("smooth_scrolling={}\n", s.smooth_scrolling));
    out.push_str(&format!("reduce_motion={}\n", escape(&s.reduce_motion)));
    out.push_str(&format!("pause_in_background={}\n", s.pause_in_background));
    out.push_str(&format!(
        "wallpaper_theme_colors={}\n",
        s.wallpaper_theme_colors
    ));
    out.push_str(&format!("wallpaper_color={}\n", escape(&s.wallpaper_color)));
    out.push_str(&format!(
        "wallpaper_folder={}\n",
        escape(&s.wallpaper_folder)
    ));
    out.push_str(&format!(
        "wallpaper_source={}\n",
        escape(&s.wallpaper_source)
    ));
    out.push_str(&format!(
        "new_thread_background_path={}\n",
        escape(&s.new_thread_background_path)
    ));
    out.push_str(&format!(
        "new_thread_background_name={}\n",
        escape(&s.new_thread_background_name)
    ));
    out.push_str(&format!(
        "new_thread_background_focal_x={}\n",
        s.new_thread_background_focal_x
    ));
    out.push_str(&format!(
        "new_thread_background_focal_y={}\n",
        s.new_thread_background_focal_y
    ));
    out.push_str(&format!(
        "new_thread_background_zoom={}\n",
        s.new_thread_background_zoom
    ));
    out.push_str(&format!(
        "new_thread_background_effect={}\n",
        escape(&s.new_thread_background_effect)
    ));
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
            // The glass trio: booleans follow the same strict
            // `true`/`false` rule as the other switches; the opacity uses
            // the theme's own clamp so a hand-edited file and the slider
            // cannot disagree about the supported range.
            "background_translucent" => {
                if let Some(b) = parse_bool(value) {
                    s.background_translucent = b;
                }
            }
            "window_opacity" => {
                if let Ok(f) = value.trim().parse::<f64>()
                    && f.is_finite()
                {
                    s.window_opacity = clamp_window_opacity(f);
                }
            }
            "sidebar_tone" => {
                if let Ok(f) = value.trim().parse::<f64>()
                    && f.is_finite()
                {
                    s.sidebar_tone = clamp_sidebar_tone(f);
                }
            }
            "sidebar_width" => {
                if let Ok(f) = value.trim().parse::<f64>()
                    && f.is_finite()
                {
                    s.sidebar_width = clamp_sidebar_width(f);
                }
            }
            "sidebar_collapsed" => {
                if let Some(b) = parse_bool(value) {
                    s.sidebar_collapsed = b;
                }
            }
            "background_blur" => {
                if let Some(b) = parse_bool(value) {
                    s.background_blur = b;
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
            // Appearance-surface keys (`appearance-parity` R7). Unknown words
            // and malformed numbers keep the default; string ids are kept as
            // written (the registry resolves/falls back at apply time).
            "theme_light" => {
                let id = unescape(value);
                if !id.is_empty() {
                    s.theme_light = id;
                }
            }
            "theme_dark" => {
                let id = unescape(value);
                if !id.is_empty() {
                    s.theme_dark = id;
                }
            }
            "ui_font_family" => {
                if let Some(family) = font_family_from_word(&unescape(value)) {
                    s.ui_font_family = font_family_word(&family);
                }
            }
            "terminal_font_family" => {
                if let Some(family) = font_family_from_word(&unescape(value)) {
                    s.terminal_font_family = font_family_word(&family);
                }
            }
            "code_font_family" => {
                if let Some(family) = font_family_from_word(&unescape(value)) {
                    s.code_font_family = font_family_word(&family);
                }
            }
            "terminal_font_size" => {
                if let Ok(f) = value.trim().parse::<f64>()
                    && f.is_finite()
                {
                    s.terminal_font_size = clamp_ui_font_size(f, TERMINAL_FONT_SIZE_DEFAULT);
                }
            }
            "code_font_size" => {
                if let Ok(f) = value.trim().parse::<f64>()
                    && f.is_finite()
                {
                    s.code_font_size = clamp_ui_font_size(f, CODE_FONT_SIZE_DEFAULT);
                }
            }
            "transcript_width" => {
                if let Ok(f) = value.trim().parse::<f64>()
                    && f.is_finite()
                {
                    s.transcript_width = normalize_transcript_width(f);
                }
            }
            "render_math" => {
                if let Some(b) = parse_bool(value) {
                    s.render_math = b;
                }
            }
            "math_font" => {
                if let Some(font) = MathFont::parse(&unescape(value)) {
                    s.math_font = font.key().to_string();
                }
            }
            "math_size" => {
                // Any number rounds and clamps into the page's ladder range;
                // text keeps the default rather than failing the file.
                if let Ok(f) = value.trim().parse::<f64>()
                    && f.is_finite()
                {
                    s.math_size = f
                        .round()
                        .clamp(f64::from(MATH_SIZE_MIN), f64::from(MATH_SIZE_MAX))
                        as u16;
                }
            }
            "smooth_scrolling" => {
                if let Some(b) = parse_bool(value) {
                    s.smooth_scrolling = b;
                }
            }
            "reduce_motion" => {
                if let Some(preference) = reduce_motion_from_word(&unescape(value)) {
                    s.reduce_motion = reduce_motion_word(preference).to_string();
                }
            }
            "pause_in_background" => {
                if let Some(b) = parse_bool(value) {
                    s.pause_in_background = b;
                }
            }
            "wallpaper_theme_colors" => {
                if let Some(b) = parse_bool(value) {
                    s.wallpaper_theme_colors = b;
                }
            }
            "wallpaper_color" => {
                if let Some(color) = wallpaper_color_from_str(&unescape(value)) {
                    s.wallpaper_color = wallpaper_color_to_str(color);
                }
            }
            "wallpaper_folder" => {
                let path = unescape(value);
                if !path.is_empty() {
                    s.wallpaper_folder = path;
                }
            }
            "wallpaper_source" => {
                let path = unescape(value);
                if !path.is_empty() {
                    s.wallpaper_source = path;
                }
            }
            "new_thread_background_path" => {
                let path = unescape(value);
                if !path.is_empty() {
                    s.new_thread_background_path = path;
                }
            }
            "new_thread_background_name" => {
                s.new_thread_background_name = unescape(value);
            }
            "new_thread_background_focal_x" => {
                if let Ok(f) = value.trim().parse::<f64>()
                    && f.is_finite()
                {
                    s.new_thread_background_focal_x = clamp_focal(f);
                }
            }
            "new_thread_background_focal_y" => {
                if let Ok(f) = value.trim().parse::<f64>()
                    && f.is_finite()
                {
                    s.new_thread_background_focal_y = clamp_focal(f);
                }
            }
            "new_thread_background_zoom" => {
                if let Ok(f) = value.trim().parse::<f64>()
                    && f.is_finite()
                {
                    s.new_thread_background_zoom = clamp_zoom(f);
                }
            }
            "new_thread_background_effect" => {
                if let Some(effect) = NewThreadBackgroundEffect::parse(&unescape(value)) {
                    s.new_thread_background_effect = effect.word().to_string();
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
///
/// The temp file is opened `O_EXCL` so a symlink planted at its path cannot
/// redirect the write, and the directory is fsynced after the rename so the
/// new name survives a power loss — renaming alone orders the data but not
/// the directory entry.
fn atomic_write(dir: &Path, name: &str, contents: &str) -> bool {
    if let Err(e) = fs::create_dir_all(dir) {
        eprintln!("persist: cannot create {}: {e}", dir.display());
        return false;
    }
    let tmp = dir.join(format!(".{name}.tmp-{}", std::process::id()));
    let target = dir.join(name);
    // A temp file orphaned by a kill was never renamed, so it holds no
    // state anyone reads. Clear it first: `create_new` would otherwise fail
    // on it forever, wedging every later save under the same pid.
    let _ = fs::remove_file(&tmp);
    let written = (|| -> io::Result<()> {
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp)?;
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
    // Persist the directory entry itself. On a crash the file contents are
    // already on disk, but without this the rename can be lost, leaving the
    // previous settings in place — a silent revert rather than data loss.
    // Best-effort: not every platform can open a directory, and the write
    // above already succeeded, so a failure here is not a failed save.
    if let Ok(d) = fs::File::open(dir) {
        let _ = d.sync_all();
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
                background_translucent: true,
                window_opacity: 62.0,
                sidebar_tone: 74.0,
                sidebar_width: 300.0,
                sidebar_collapsed: true,
                background_blur: true,
                launch_at_login: true,
                notifications_enabled: false,
                username: "tinhpt".into(),
                display_name: "Tinh Pham".into(),
                channel_email: false,
                channel_desktop: true,
                channel_digest: true,
                digest: "weekly".into(),
                timezone: "Asia/Tokyo".into(),
                ..StoredSettings::default()
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
            background_translucent: true,
            window_opacity: 55.0,
            sidebar_tone: 55.0,
            sidebar_width: 300.0,
            sidebar_collapsed: true,
            background_blur: false,
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
            ..StoredSettings::default()
        };
        let text = serialize(&s);
        // Hostile bytes must not create extra lines: marker + 43 keys.
        assert_eq!(text.lines().count(), 44);
        assert_eq!(parse(&text), Some(s));
        // Empty strings round-trip too.
        let empty = StoredSettings::default();
        assert_eq!(parse(&serialize(&empty)), Some(empty));
    }

    /// The glass trio: defaults on absence, strict booleans, clamped opacity,
    /// and every key is written back.
    #[test]
    fn glass_settings_round_trip_and_validate() {
        let written = StoredSettings {
            background_translucent: true,
            window_opacity: 55.0,
            sidebar_tone: 65.0,
            sidebar_width: 275.0,
            sidebar_collapsed: true,
            background_blur: true,
            ..StoredSettings::default()
        };
        assert_eq!(parse(&serialize(&written)), Some(written.clone()));

        let parsed = |body: &str| parse(&format!("{SETTINGS_MARKER}\n{body}")).unwrap();
        // A file written before the keys existed keeps the opaque defaults.
        let legacy = parsed("font_size=18\n");
        assert!(!legacy.background_translucent);
        assert_eq!(legacy.window_opacity, WINDOW_OPACITY_DEFAULT);
        assert_eq!(legacy.sidebar_tone, SIDEBAR_TONE_DEFAULT);
        assert_eq!(legacy.sidebar_width, SIDEBAR_WIDTH_DEFAULT);
        // The collapse key is additive: a file written before it keeps open.
        assert!(!legacy.sidebar_collapsed);
        // The pre-release pane keys are dropped, not migrated: a file that
        // still carries them keeps the shipped defaults.
        let stale = parsed("sidebar_opacity=10\npane_opacity=10\n");
        assert_eq!(stale.window_opacity, WINDOW_OPACITY_DEFAULT);
        assert_eq!(stale.sidebar_tone, SIDEBAR_TONE_DEFAULT);
        assert!(!legacy.background_blur);

        // The opacity clamps into the supported range; non-finite and
        // malformed values keep the default.
        assert_eq!(parsed("window_opacity=5\n").window_opacity, 40.0);
        assert_eq!(parsed("window_opacity=500\n").window_opacity, 100.0);
        assert_eq!(
            parsed("window_opacity=NaN\n").window_opacity,
            WINDOW_OPACITY_DEFAULT
        );
        assert_eq!(
            parsed("window_opacity=inf\n").window_opacity,
            WINDOW_OPACITY_DEFAULT
        );
        assert_eq!(
            parsed("window_opacity=banana\n").window_opacity,
            WINDOW_OPACITY_DEFAULT
        );
        // The tone is 0–100 (0 = no ink) and falls back to 50% on junk.
        assert_eq!(parsed("sidebar_tone=0\n").sidebar_tone, 0.0);
        assert_eq!(parsed("sidebar_tone=500\n").sidebar_tone, 100.0);
        assert_eq!(
            parsed("sidebar_tone=NaN\n").sidebar_tone,
            SIDEBAR_TONE_DEFAULT
        );
        assert_eq!(
            parsed("sidebar_tone=banana\n").sidebar_tone,
            SIDEBAR_TONE_DEFAULT
        );
        // The rail width clamps to 242–400 and keeps 256 on junk.
        assert_eq!(parsed("sidebar_width=10\n").sidebar_width, 242.0);
        assert_eq!(parsed("sidebar_width=9999\n").sidebar_width, 400.0);
        assert_eq!(
            parsed("sidebar_width=banana\n").sidebar_width,
            SIDEBAR_WIDTH_DEFAULT
        );
        // Booleans are strict: junk keeps the default.
        assert!(!parsed("background_blur=maybe\n").background_blur);
        assert!(!parsed("background_translucent=1\n").background_translucent);
        assert!(parsed("background_translucent=true\n").background_translucent);
        assert!(!parsed("sidebar_collapsed=maybe\n").sidebar_collapsed);
        assert!(!parsed("sidebar_collapsed=1\n").sidebar_collapsed);
        assert!(parsed("sidebar_collapsed=true\n").sidebar_collapsed);
        assert!(!parsed("sidebar_collapsed=false\n").sidebar_collapsed);

        // The serialized file carries all three keys. An older build ignores
        // unknown keys, so this stays forward/backward compatible.
        let text = serialize(&written);
        assert!(text.contains("background_translucent=true\n"), "{text}");
        assert!(text.contains("window_opacity=55\n"), "{text}");
        assert!(text.contains("sidebar_tone=65\n"), "{text}");
        assert!(text.contains("sidebar_width=275\n"), "{text}");
        assert!(text.contains("sidebar_collapsed=true\n"), "{text}");
        assert!(text.contains("background_blur=true\n"), "{text}");
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

    /// `atomic_write` must survive a temp file left behind by a kill: with
    /// `create_new` that orphan would otherwise block every later save under the
    /// same pid, turning one interrupted save into permanent silence.
    #[test]
    fn atomic_write_replaces_an_orphaned_temp_file() {
        let dir = tempdir("orphan");
        let _g = Cleanup(dir.clone());
        let orphan = dir.join(format!(".x.conf.tmp-{}", std::process::id()));
        fs::write(&orphan, "stale").unwrap();

        assert!(atomic_write(&dir, "x.conf", "fresh"));
        assert_eq!(fs::read_to_string(dir.join("x.conf")).unwrap(), "fresh");
        assert!(!orphan.exists(), "the orphan must be cleared, not left");
    }

    /// Plan NG4 freezes the persisted keys: the same names, in the same order,
    /// one `key=value` line each. A rename or reorder here is a silent config
    /// migration for every existing install.
    #[test]
    fn serialized_keys_are_frozen_in_order() {
        let text = serialize(&StoredSettings::default());
        let keys: Vec<&str> = text
            .lines()
            .skip(1)
            .map(|line| line.split_once('=').expect("key=value line").0)
            .collect();
        assert_eq!(
            keys,
            [
                "accent",
                "appearance",
                "language",
                "font_size",
                "background_translucent",
                "window_opacity",
                "sidebar_tone",
                "sidebar_width",
                "sidebar_collapsed",
                "background_blur",
                "launch_at_login",
                "notifications_enabled",
                "username",
                "display_name",
                "channel_email",
                "channel_desktop",
                "channel_digest",
                "digest",
                "timezone",
                // Appearance-surface keys (`appearance-parity` R7).
                "theme_light",
                "theme_dark",
                "ui_font_family",
                "terminal_font_family",
                "code_font_family",
                "terminal_font_size",
                "code_font_size",
                "transcript_width",
                "render_math",
                "math_font",
                "math_size",
                "smooth_scrolling",
                "reduce_motion",
                "pause_in_background",
                "wallpaper_theme_colors",
                "wallpaper_color",
                "wallpaper_folder",
                "wallpaper_source",
                "new_thread_background_path",
                "new_thread_background_name",
                "new_thread_background_focal_x",
                "new_thread_background_focal_y",
                "new_thread_background_zoom",
                "new_thread_background_effect",
            ]
        );
        // Marker + 18 original keys + 24 appearance keys + sidebar_collapsed.
        assert_eq!(keys.len(), 43);
        assert_eq!(text.lines().count(), 44);
        assert_eq!(text.lines().next(), Some(SETTINGS_MARKER));
    }

    /// The full app-state round trip the settings pages rely on:
    /// `AppSettings` → stored → file text → parse → `AppSettings` → stored.
    #[test]
    fn app_settings_round_trip_through_the_file() {
        use crate::app_settings::AppSettings;

        // The default state, through the real codec.
        let stored = AppSettings::default().to_stored();
        let parsed = parse(&serialize(&stored)).expect("default file is managed");
        let reloaded = AppSettings::from_stored(parsed);
        assert_eq!(reloaded.to_stored(), stored);

        // A fully populated state. The accent is green because the HSLA codec
        // is lossy for some channels (see `app_settings::tests`); everything
        // else must survive byte-exact.
        let populated = StoredSettings {
            accent: "#1F7A45".into(),
            appearance: "system".into(),
            language: "vi".into(),
            font_size: 11.0,
            background_translucent: true,
            window_opacity: 62.0,
            sidebar_tone: 74.0,
            sidebar_width: 300.0,
            sidebar_collapsed: true,
            background_blur: true,
            launch_at_login: true,
            notifications_enabled: false,
            username: "tinhpt".into(),
            display_name: "Tinh Pham".into(),
            channel_email: false,
            channel_desktop: true,
            channel_digest: true,
            digest: "weekly".into(),
            timezone: "Asia/Tokyo".into(),
            theme_light: "custom-light".into(),
            theme_dark: "custom-dark".into(),
            ui_font_family: "installed:Inter".into(),
            terminal_font_family: "xanh-mono".into(),
            code_font_family: "google-sans-code".into(),
            terminal_font_size: 15.0,
            code_font_size: 11.0,
            transcript_width: 800.0,
            render_math: false,
            math_font: "stixTwo".into(),
            math_size: 120,
            smooth_scrolling: false,
            reduce_motion: "on".into(),
            pause_in_background: true,
            wallpaper_theme_colors: true,
            wallpaper_color: "#AABBCC".into(),
            wallpaper_folder: "/Users/tinh/Wallpapers/=set".into(),
            wallpaper_source: "/Users/tinh/Wallpapers/=set/a b.png".into(),
            new_thread_background_path:
                "/Users/tinh/Library/Application Support/mono-desk/new-thread-backgrounds/bg image.png"
                    .into(),
            new_thread_background_name: "bg image.png".into(),
            new_thread_background_focal_x: 0.25,
            new_thread_background_focal_y: 0.75,
            new_thread_background_zoom: 2.0,
            new_thread_background_effect: "dither".into(),
        };
        let parsed = parse(&serialize(&populated)).expect("populated file is managed");
        let settings = AppSettings::from_stored(parsed);
        assert_eq!(settings.to_stored(), populated);
    }

    /// The persisted accent keeps its `#RRGGBB` shape across the app-state
    /// hop, and an unparseable stored accent falls back to "theme's own"
    /// (empty) rather than being copied through.
    #[test]
    fn accent_survives_the_app_state_hop() {
        use crate::app_settings::AppSettings;

        let green = StoredSettings {
            accent: "#1F7A45".into(),
            ..StoredSettings::default()
        };
        let settings = AppSettings::from_stored(green.clone());
        assert_eq!(settings.to_stored(), green);

        let junk = StoredSettings {
            accent: "not-a-color".into(),
            ..StoredSettings::default()
        };
        assert_eq!(
            AppSettings::from_stored(junk).to_stored().accent,
            String::new()
        );
    }

    /// The 24 appearance keys round-trip through the codec, including an
    /// `installed:` family word and paths holding spaces, `=`, a backslash and
    /// an escaped newline.
    #[test]
    fn appearance_keys_round_trip_with_hostile_words_and_paths() {
        let s = StoredSettings {
            theme_light: "custom:light".into(),
            theme_dark: "custom:dark".into(),
            ui_font_family: "installed:My = Font".into(),
            terminal_font_family: "installed:My = Font".into(),
            code_font_family: "installed:日本語 Mono".into(),
            terminal_font_size: 15.0,
            code_font_size: 11.0,
            transcript_width: 800.0,
            render_math: false,
            math_font: "euler".into(),
            math_size: 120,
            smooth_scrolling: false,
            reduce_motion: "on".into(),
            pause_in_background: true,
            wallpaper_theme_colors: true,
            wallpaper_color: "#AABBCC".into(),
            wallpaper_folder: "C:\\Users\\tinh\\Wallpapers = set".into(),
            wallpaper_source: "/tmp/a=b/c d.png".into(),
            new_thread_background_path: "/tmp/bg\nsecond line.png".into(),
            new_thread_background_name: "na=me\n.png".into(),
            new_thread_background_focal_x: 0.25,
            new_thread_background_focal_y: 0.75,
            new_thread_background_zoom: 2.0,
            new_thread_background_effect: "halftone".into(),
            ..StoredSettings::default()
        };
        let text = serialize(&s);
        // Hostile bytes must not create extra lines: marker + 43 keys.
        assert_eq!(text.lines().count(), 44, "{text}");
        assert_eq!(parse(&text), Some(s));
    }

    /// A file carrying every new key round-trips byte-for-byte through
    /// serialize → parse → serialize, so a save after a load cannot churn the
    /// file.
    #[test]
    fn appearance_key_file_round_trips_byte_equal() {
        let s = StoredSettings {
            theme_light: "custom:light".into(),
            theme_dark: "custom:dark".into(),
            ui_font_family: "installed:Inter".into(),
            terminal_font_family: "installed:My Font".into(),
            code_font_family: "xanh-mono".into(),
            terminal_font_size: 15.0,
            code_font_size: 11.5,
            transcript_width: 800.0,
            render_math: false,
            math_font: "euler".into(),
            math_size: 135,
            smooth_scrolling: false,
            reduce_motion: "off".into(),
            pause_in_background: true,
            wallpaper_theme_colors: true,
            wallpaper_color: "#A1B2C3".into(),
            wallpaper_folder: "/Users/tinh/Wallpapers".into(),
            wallpaper_source: "/Users/tinh/Wallpapers/a b.png".into(),
            new_thread_background_path: "/Users/tinh/bg image.png".into(),
            new_thread_background_name: "bg image.png".into(),
            new_thread_background_focal_x: 0.25,
            new_thread_background_focal_y: 0.75,
            new_thread_background_zoom: 2.0,
            new_thread_background_effect: "scanlines".into(),
            ..StoredSettings::default()
        };
        let text = serialize(&s);
        assert_eq!(text.lines().count(), 44, "{text}");
        let parsed = parse(&text).expect("managed file");
        assert_eq!(parsed, s);
        assert_eq!(serialize(&parsed), text);
    }

    /// Unknown words, empty strings and non-finite/malformed numbers keep each
    /// new key's default; a hand-edited file never fails to load because of
    /// them.
    #[test]
    fn invalid_appearance_words_and_numbers_fall_back() {
        let text = format!(
            "{SETTINGS_MARKER}\n\
             theme_light=\n\
             theme_dark=\n\
             ui_font_family=bogus\n\
             terminal_font_family=installed:\n\
             code_font_family=\n\
             terminal_font_size=NaN\n\
             code_font_size=inf\n\
             transcript_width=NaN\n\
             render_math=maybe\n\
             math_font=bogus\n\
             math_size=banana\n\
             smooth_scrolling=1\n\
             reduce_motion=bogus\n\
             pause_in_background=yes\n\
             wallpaper_theme_colors=1\n\
             wallpaper_color=not-a-color\n\
             wallpaper_folder=\n\
             wallpaper_source=\n\
             new_thread_background_path=\n\
             new_thread_background_name=\n\
             new_thread_background_focal_x=NaN\n\
             new_thread_background_focal_y=inf\n\
             new_thread_background_zoom=NaN\n\
             new_thread_background_effect=bogus\n"
        );
        assert_eq!(parse(&text), Some(StoredSettings::default()));
    }

    /// Numeric appearance keys clamp like their controls do.
    #[test]
    fn appearance_numbers_clamp_like_the_controls() {
        let parsed = |body: &str| parse(&format!("{SETTINGS_MARKER}\n{body}")).unwrap();

        let high = parsed(
            "terminal_font_size=99\ncode_font_size=99\ntranscript_width=9999\n             math_size=999\nnew_thread_background_focal_x=5\n             new_thread_background_focal_y=5\nnew_thread_background_zoom=9\n",
        );
        assert_eq!(high.terminal_font_size, 32.0);
        assert_eq!(high.code_font_size, 32.0);
        assert_eq!(high.transcript_width, 1200.0);
        assert_eq!(high.math_size, 150);
        assert_eq!(high.new_thread_background_focal_x, 1.0);
        assert_eq!(high.new_thread_background_focal_y, 1.0);
        assert_eq!(high.new_thread_background_zoom, 4.0);

        let low = parsed(
            "terminal_font_size=1\ncode_font_size=1\ntranscript_width=1\n             math_size=1\nnew_thread_background_focal_x=-2\n             new_thread_background_focal_y=-2\nnew_thread_background_zoom=0.5\n",
        );
        assert_eq!(low.terminal_font_size, 8.0);
        assert_eq!(low.code_font_size, 8.0);
        assert_eq!(low.transcript_width, 560.0);
        assert_eq!(low.math_size, 90);
        assert_eq!(low.new_thread_background_focal_x, 0.0);
        assert_eq!(low.new_thread_background_focal_y, 0.0);
        assert_eq!(low.new_thread_background_zoom, 1.0);

        // On the grid: a 736 px width and a 90–150 math size stay put.
        let exact = parsed("transcript_width=736\nmath_size=100\n");
        assert_eq!(exact.transcript_width, 736.0);
        assert_eq!(exact.math_size, 100);
    }

    /// A settings file written before this phase (the 18 original keys) loads
    /// unchanged, with every new key taking its default.
    #[test]
    fn old_eighteen_key_file_loads_with_new_defaults() {
        let old = format!(
            "{SETTINGS_MARKER}\n\
             accent=\n\
             appearance=light\n\
             language=en\n\
             font_size=14\n\
             background_translucent=false\n\
             window_opacity=85\n\
             sidebar_tone=50\n\
             sidebar_width=256\n\
             background_blur=false\n\
             launch_at_login=false\n\
             notifications_enabled=true\n\
             username=ninja\n\
             display_name=Ninja\n\
             channel_email=true\n\
             channel_desktop=true\n\
             channel_digest=false\n\
             digest=immediate\n\
             timezone=system\n"
        );
        assert_eq!(old.lines().count(), 19);
        assert_eq!(parse(&old), Some(StoredSettings::default()));
    }
}
