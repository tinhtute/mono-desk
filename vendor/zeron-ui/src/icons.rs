//! Embedded icon assets + the gpui [`AssetSource`] that serves them.
//!
//! The UI glyph set is **Phosphor Icons** v2.1.1 (https://phosphoricons.com/),
//! MIT licensed — SVGs copied verbatim from
//! `@phosphor-icons/core@2.1.1/assets/regular/`, except the two filled twin
//! slots ([`FAST_TIER_BOLD`], [`STAR_BOLD`]) which take the `fill` weight.
//! Phosphor draws on a 256-unit canvas with a 16-unit outline, i.e. the same
//! 1.5-on-24 stroke weight as the Solar Linear set it replaced.
//!
//! Asset filenames keep zeron's slot names so the vendored call sites stay
//! untouched; the `// phosphor:` marker on each line records which Phosphor
//! glyph fills that slot, and `docs/memory/zeron-stack-port.md` keeps the
//! swap record. The brand marks (`*_MARK`, [`ZERON_LOGO`]) are the documented
//! exception: they are logos ported from zeron's `icons.tsx`, not UI glyphs,
//! and Phosphor ships no equivalent.
//!
//! Icons render via [`icon`]: `icon(icons::PAPERCLIP).size(px(16.)).text_color(…)`.
//!
//! Swapping the artwork is safe because gpui rasterizes an SVG into an alpha
//! mask and tints it with the element's `text_color` — the `fill` colour in
//! the file never reaches the screen.

use std::borrow::Cow;

use gpui::{
    AssetSource, Div, Hsla, ParentElement as _, Result, SharedString, Styled as _, Svg, div, px,
    svg,
};

macro_rules! icon_assets {
    ($(($const_name:ident, $path:literal)),+ $(,)?) => {
        $(pub const $const_name: &str = concat!("icons/", $path, ".svg");)+

        /// Serves the embedded control icons to gpui's SVG renderer.
        struct ControlAssets;

        impl AssetSource for ControlAssets {
            fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
                Ok(match path {
                    $(concat!("icons/", $path, ".svg") => Some(Cow::Borrowed(
                        include_bytes!(concat!("../assets/icons/", $path, ".svg")).as_slice(),
                    )),)+
                    _ => None,
                })
            }

            fn list(&self, path: &str) -> Result<Vec<SharedString>> {
                let all = [$(concat!("icons/", $path, ".svg")),+];
                Ok(all
                    .iter()
                    .filter(|p| p.starts_with(path))
                    .map(|p| SharedString::from(*p))
                    .collect())
            }
        }
    };
}

icon_assets![
    (PROJECT_DEFAULT, "project-default"), // phosphor: folder-simple
    (REMOTE_SERVER, "remote-server"), // phosphor: database
    (FAST_TIER, "fast-tier"), // phosphor: lightning
    (FAST_TIER_BOLD, "fast-tier-bold"), // phosphor: fill/lightning-fill
    (MONITOR, "monitor"), // phosphor: monitor
    (SUN, "sun"), // phosphor: sun
    (MOON, "moon"), // phosphor: moon
    (GLOBE, "globe"), // phosphor: globe
    (CURSOR_SQUARE, "cursor-square"), // phosphor: selection
    (CURSOR, "cursor"), // phosphor: cursor
    (LAPTOP, "laptop"), // phosphor: laptop
    (PEN_NEW_SQUARE, "pen-new-square"), // phosphor: note-pencil
    (SORT, "sort"), // phosphor: arrows-down-up
    (MORE_HORIZONTAL, "more-horizontal"), // phosphor: dots-three
    (SORT_VERTICAL, "sort-vertical"), // phosphor: arrows-left-right
    (DRAG_HANDLE, "drag-handle"), // phosphor: dots-six-vertical
    (QUEUE_DRAG_HANDLE, "queue-drag-handle"), // phosphor: dots-six-vertical
    (QUEUE_SEND, "queue-send"), // phosphor: paper-plane-tilt
    (QUEUE_CHECK, "queue-check"), // phosphor: check-circle
    (QUEUE_CLOSE, "queue-close"), // phosphor: x-circle
    (QUEUE_PAPERCLIP, "queue-paperclip"), // phosphor: paperclip
    (CLOCK_CIRCLE, "clock-circle"), // phosphor: clock
    (CALENDAR, "calendar"), // phosphor: calendar-blank
    (LIST, "list"), // phosphor: list-bullets
    (FOLDER_WITH_FILES, "folder-with-files"), // phosphor: folder-open
    (FILE_TREE, "file-tree"), // phosphor: tree-view
    (FOLDER_TREE, "folder-tree"), // phosphor: folders
    (FLOPPY_DISK, "floppy-disk"), // phosphor: floppy-disk
    (FOLDER, "folder"), // phosphor: folder
    (FORK, "fork"), // phosphor: git-fork
    (GIT_BRANCH, "git-branch"), // phosphor: git-branch
    (PULL_REQUEST, "pull-request"), // phosphor: git-pull-request
    (WORKTREE, "worktree"), // phosphor: tree-structure
    (PLUS, "plus"), // phosphor: plus
    (MINUS, "minus"), // phosphor: minus
    (ARROW_LEFT, "arrow-left"), // phosphor: arrow-left
    (ARROW_RIGHT, "arrow-right"), // phosphor: arrow-right
    (ALT_ARROW_DOWN, "alt-arrow-down"), // phosphor: caret-down
    (ALT_ARROW_UP, "alt-arrow-up"), // phosphor: caret-up
    (ALT_ARROW_LEFT, "alt-arrow-left"), // phosphor: caret-left
    (ALT_ARROW_RIGHT, "alt-arrow-right"), // phosphor: caret-right
    (PULL_REQUEST_DRAFT, "pull-request-draft"), // phosphor: git-pull-request
    (CLOUD, "cloud"), // phosphor: cloud
    (TAG, "tag"), // phosphor: tag
    (KEY_MINIMALISTIC, "key-minimalistic"), // phosphor: key
    (KEYBOARD, "keyboard"), // phosphor: keyboard
    (ARROW_UP, "arrow-up"), // phosphor: arrow-up
    (ARROW_DOWN, "arrow-down"), // phosphor: arrow-down
    (ARROW_UP_RIGHT, "arrow-up-right"), // phosphor: arrow-up-right
    (RETURN, "return"), // phosphor: arrow-elbow-down-left
    (EXPAND_ARROWS, "expand-arrows"), // phosphor: arrows-out
    (COLLAPSE_ARROWS, "collapse-arrows"), // phosphor: arrows-in
    (FOLD_VERTICAL, "fold-vertical"), // phosphor: arrows-in-line-vertical
    (SPLIT_COLUMNS, "split-columns"), // phosphor: columns
    (WRAP_TEXT, "wrap-text"), // phosphor: text-align-left
    (SMARTPHONE, "smartphone"), // phosphor: device-mobile
    (ARCHIVE_UP_MINIMALISTIC, "archive-up-minimalistic"), // phosphor: tray-arrow-up
    (REFRESH, "refresh"), // phosphor: arrows-clockwise
    (RESTART, "restart"), // phosphor: arrow-counter-clockwise
    (ADD_CIRCLE, "add-circle"), // phosphor: plus-circle
    (TUNING, "tuning"), // phosphor: sliders-horizontal
    (EYE, "eye"), // phosphor: eye
    (EYE_CLOSED, "eye-closed"), // phosphor: eye-closed
    (PAPERCLIP, "paperclip"), // phosphor: paperclip
    (PIN, "pin"), // phosphor: push-pin
    (PEN, "pen"), // phosphor: pencil
    (ARCHIVE_MINIMALISTIC, "archive-minimalistic"), // phosphor: archive
    (TRASH_BIN_MINIMALISTIC, "trash-bin-minimalistic"), // phosphor: trash
    (SETTINGS, "settings"), // phosphor: sliders
    (SETTINGS_MINIMALISTIC, "settings-minimalistic"), // phosphor: gear
    (LOGOUT_2, "logout-2"), // phosphor: sign-out
    (MAGNIFER, "magnifer"), // phosphor: magnifying-glass
    (PALETTE_SEARCH, "palette-search"), // phosphor: magnifying-glass
    (COMMAND, "command"), // phosphor: command
    (DOCUMENT, "document"), // phosphor: file-text
    (DOCUMENT_ADD, "document-add"), // phosphor: file-plus
    (FILE_CODE, "file-code"), // phosphor: file-code
    (FILE_STYLE, "file-style"), // phosphor: file-css
    (FILE_DATA, "file-data"), // phosphor: file-csv
    (FILE_MARKDOWN, "file-markdown"), // phosphor: file-md
    (FILE_IMAGE, "file-image"), // phosphor: file-image
    (GALLERY, "gallery"), // phosphor: images
    (GLOBAL, "global"), // phosphor: globe-hemisphere-west
    (CHECKLIST, "checklist"), // phosphor: list-checks
    (WIDGET, "widget"), // phosphor: squares-four
    (MAGIC_STICK_3, "magic-stick-3"), // phosphor: magic-wand
    (WIFI_OFF, "wifi-off"), // phosphor: wifi-slash
    (CLOSE_CIRCLE, "close-circle"), // phosphor: x-circle
    (INFO_CIRCLE, "info-circle"), // phosphor: info
    (DANGER_TRIANGLE, "danger-triangle"), // phosphor: warning
    (CHAT_ROUND_LINE, "chat-round-line"), // phosphor: chat-circle
    (BOT, "bot"), // phosphor: robot
    (BELL, "bell"), // phosphor: bell
    (USER, "user"), // phosphor: user
    (VOLUME_LOUD, "volume-loud"), // phosphor: speaker-high
    (MICROPHONE, "microphone"), // phosphor: microphone
    (MICROPHONE_OFF, "microphone-off"), // phosphor: microphone-slash
    (PHONE_HANG_UP, "phone-hang-up"), // phosphor: phone-x
    (TERMINAL, "terminal"), // phosphor: terminal-window
    (CLOSE, "close"), // phosphor: x
    (WINDOW_MINIMIZE, "window-minimize"), // phosphor: minus
    (WINDOW_MAXIMIZE, "window-maximize"), // phosphor: square
    (WINDOW_RESTORE, "window-restore"), // phosphor: copy
    (HARD_DRIVE, "hard-drive"), // phosphor: hard-drives
    (HOME, "home"), // phosphor: house
    (STOP, "stop"), // phosphor: square
    (CHECK, "check"), // phosphor: check
    (COPY, "copy"), // phosphor: copy
    (ACTION_PLAY, "action-play"), // phosphor: play
    (ACTION_TEST, "action-test"), // phosphor: test-tube
    (ACTION_LINT, "action-lint"), // phosphor: check-circle
    (ACTION_CONFIGURE, "action-configure"), // phosphor: gear
    (ACTION_BUILD, "action-build"), // phosphor: hammer
    (ACTION_DEBUG, "action-debug"), // phosphor: bug
    (STAR, "star"), // phosphor: star
    (STAR_BOLD, "star-bold"), // phosphor: fill/star-fill
    (ZERON_LOGO, "zeron-logo"), // brand mark: zeron artwork, no Phosphor equivalent
    (CLAUDE_MARK, "claude-mark"), // brand mark: zeron artwork, no Phosphor equivalent
    (OPENAI_MARK, "openai-mark"), // brand mark: zeron artwork, no Phosphor equivalent
    (CURSOR_MARK, "cursor-mark"), // brand mark: zeron artwork, no Phosphor equivalent
    (DEVIN_MARK, "devin-mark"), // brand mark: zeron artwork, no Phosphor equivalent
    (GROK_MARK, "grok-mark"), // brand mark: zeron artwork, no Phosphor equivalent
    (HERMES_MARK, "hermes-mark"), // brand mark: zeron artwork, no Phosphor equivalent
    (PI_MARK, "pi-mark"), // brand mark: zeron artwork, no Phosphor equivalent
    (OPENCODE_MARK, "opencode-mark"), // brand mark: zeron artwork, no Phosphor equivalent
    (GITHUB_MARK, "github-mark"), // phosphor: github-logo
    (WALLET, "wallet"), // phosphor: wallet
    (ANTIGRAVITY_MARK, "antigravity-mark"), // brand mark: zeron artwork, no Phosphor equivalent
    (MUSE_MARK, "muse-mark"), // brand mark: zeron artwork, no Phosphor equivalent
];

/// Serves the compact control-icon set through the single asset source
/// registered at app startup. zeron's file-identity icon theme
/// (`file_icons.rs`) is not ported, so no other path resolves.
pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        ControlAssets.load(path)
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        ControlAssets.list(path)
    }
}

/// The Claude mark's brand orange (`#D97757`) — zeron keeps it even on the
/// monochrome surface.
pub fn claude_brand() -> Hsla {
    gpui::rgb(0xD97757).into()
}

/// The Muse Code mark's brand blue (`#0081FB`), kept like Claude's orange.
pub fn muse_brand() -> Hsla {
    gpui::rgb(0x0081FB).into()
}

/// An icon element for an embedded asset path. Size and colour are set by the
/// caller (`.size(..)`, `.text_color(..)`), matching the web app's
/// `[&_svg]:size-4` idiom. GPUI rasterizes the SVG into an alpha mask and
/// tints it with this element's `text_color`, so always set `.text_color(..)`
/// on the icon itself — a color on its parent does not tint it.
pub fn icon(path: &'static str) -> Svg {
    svg().path(path).flex_none()
}

/// The Zeron Icons sidebar glyph (icons.zeron.sh "Sidebar" / "Right
/// sidebar"): a rounded frame holding a panel whose width morphs 5.5 → 1.75
/// as the sidebar closes. gpui SVGs are static, so it is drawn from quads in
/// the source's 24-unit space, scaled to `size`. `open` is the morph progress
/// (1 = open; drive it with [`crate::motion::state_t`]); `right` mirrors it.
pub fn sidebar_glyph(open: f32, right: bool, size: f32, color: Hsla) -> Div {
    let s = size / 24.0;
    // `<rect x=3 y=4 w=18 h=16 rx=4 stroke-width=1.75>`: the centered stroke
    // grows the box by half the stroke on each side.
    let stroke = 1.75;
    let frame = div()
        .absolute()
        .left(px((3.0 - stroke / 2.0) * s))
        .top(px((4.0 - stroke / 2.0) * s))
        .w(px((18.0 + stroke) * s))
        .h(px((16.0 + stroke) * s))
        .rounded(px((4.0 + stroke / 2.0) * s))
        .border(px(stroke * s))
        .border_color(color);
    // `<rect class=zi-panel x=6.5 y=7.5 w=5.5 h=9 rx=.875>`; closed: w=1.75.
    let inset = px(6.5 * s);
    let panel = div()
        .absolute()
        .top(px(7.5 * s))
        .w(px(crate::motion::lerp(1.75, 5.5, open.clamp(0.0, 1.0)) * s))
        .h(px(9.0 * s))
        .rounded(px(0.875 * s))
        .bg(color);
    let panel = if right {
        panel.right(inset)
    } else {
        panel.left(inset)
    };
    div()
        .relative()
        .flex_none()
        .size(px(size))
        .child(frame)
        .child(panel)
}

/// The frame every panel toggle shares (`<rect x=3 y=4 w=18 h=16 rx=4
/// stroke-width=1.75>`), from quads so the strokes stay crisp at 16px.
fn panel_frame(s: f32, color: Hsla) -> Div {
    let stroke = 1.75;
    div()
        .absolute()
        .left(px((3.0 - stroke / 2.0) * s))
        .top(px((4.0 - stroke / 2.0) * s))
        .w(px((18.0 + stroke) * s))
        .h(px((16.0 + stroke) * s))
        .rounded(px((4.0 + stroke / 2.0) * s))
        .border(px(stroke * s))
        .border_color(color)
}

/// The session card toggle: [`sidebar_glyph`]'s frame with a panel in only
/// its top-right corner, drawn from the same quads so the strokes match the
/// sidebar toggles beside it.
pub fn corner_card_glyph(size: f32, color: Hsla) -> Div {
    let s = size / 24.0;
    let frame = panel_frame(s, color);
    // The open sidebar panel's footprint (x 12..17.5 from the right inset),
    // cut to its top half.
    let panel = div()
        .absolute()
        .top(px(7.5 * s))
        .right(px(6.5 * s))
        .w(px(5.5 * s))
        .h(px(4.5 * s))
        .rounded(px(0.875 * s))
        .bg(color);
    div()
        .relative()
        .flex_none()
        .size(px(size))
        .child(frame)
        .child(panel)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Slots that intentionally keep zeron's own artwork: the harness and app
    /// brand marks are logos, not UI glyphs, and Phosphor ships no equivalent.
    const BRAND_MARKS: &[&str] = &[
        "icons/antigravity-mark.svg",
        "icons/claude-mark.svg",
        "icons/cursor-mark.svg",
        "icons/devin-mark.svg",
        "icons/grok-mark.svg",
        "icons/hermes-mark.svg",
        "icons/muse-mark.svg",
        "icons/openai-mark.svg",
        "icons/opencode-mark.svg",
        "icons/pi-mark.svg",
        "icons/zeron-logo.svg",
    ];

    /// The UI set is Phosphor Icons (https://phosphoricons.com/, MIT): every
    /// slot that is not a brand mark carries a Phosphor glyph on Phosphor's
    /// 256-unit canvas. Red before the swap (the Solar and hand-drawn ports
    /// use 16/24-unit canvases), green after it.
    #[test]
    fn ui_icons_are_phosphor_glyphs() {
        let paths = Assets.list("icons/").expect("icon asset source lists");
        assert!(!paths.is_empty(), "the icon asset source serves icons");
        for path in paths {
            let path = path.to_string();
            let svg = Assets
                .load(&path)
                .unwrap_or_else(|err| panic!("{path} loads: {err}"))
                .unwrap_or_else(|| panic!("{path} is embedded"));
            let svg = std::str::from_utf8(&svg).expect("icon SVG is UTF-8");
            assert!(svg.trim_start().starts_with("<svg"), "{path} is an SVG");
            if BRAND_MARKS.contains(&path.as_str()) {
                continue;
            }
            assert!(
                svg.contains("viewBox=\"0 0 256 256\""),
                "{path} is not a Phosphor glyph (expected the 256-unit canvas)"
            );
        }
    }
}
