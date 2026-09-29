//! Zero-dependency translation table for the settings-demo app.
//!
//! Supported locales: `"en"`, `"vi"`, `"zh-CN"`, `"ja"`. Brand and product
//! names (GPUI Kit, gpui-kit.com, settings-demo, Longbridge) are intentionally
//! left untranslated in every locale.

/// One row per UI string: `(key, en, vi, zh-CN, ja)`.
static STRINGS: &[(&str, &str, &str, &str, &str)] = &[
    // Pages
    ("page.general", "General", "Chung", "通用", "一般"),
    (
        "page.notifications",
        "Notifications",
        "Thông báo",
        "通知",
        "通知",
    ),
    (
        "page.profile",
        "Profile",
        "Hồ sơ",
        "个人资料",
        "プロフィール",
    ),
    ("page.about", "About", "Giới thiệu", "关于", "情報"),
    // Groups
    (
        "group.appearance",
        "Appearance",
        "Giao diện",
        "外观",
        "外観",
    ),
    ("group.startup", "Startup", "Khởi động", "启动", "起動"),
    ("group.delivery", "Delivery", "Nhận", "送达", "配信"),
    ("group.actions", "Actions", "Thao tác", "操作", "アクション"),
    (
        "group.account",
        "Account",
        "Tài khoản",
        "账户",
        "アカウント",
    ),
    ("group.links", "Links", "Liên kết", "链接", "リンク"),
    // Items
    (
        "item.dark_mode",
        "Dark Mode",
        "Chế độ tối",
        "深色模式",
        "ダークモード",
    ),
    (
        "item.dark_mode.desc",
        "Switch between light and dark themes.",
        "Chuyển đổi giữa giao diện sáng và tối.",
        "在浅色和深色主题之间切换。",
        "ライトテーマとダークテーマを切り替えます。",
    ),
    ("item.language", "Language", "Ngôn ngữ", "语言", "言語"),
    (
        "item.language.desc",
        "Interface language.",
        "Ngôn ngữ giao diện.",
        "界面语言。",
        "インターフェースの言語。",
    ),
    (
        "item.font_size",
        "Font Size",
        "Cỡ chữ",
        "字体大小",
        "フォントサイズ",
    ),
    (
        "item.font_size.desc",
        "Base UI font size, 8–32.",
        "Cỡ chữ giao diện cơ bản, 8–32.",
        "基础界面字体大小，8–32。",
        "基本UIフォントサイズ（8〜32）。",
    ),
    (
        "item.launch_at_login",
        "Launch at Login",
        "Khởi chạy khi đăng nhập",
        "登录时启动",
        "ログイン時に起動",
    ),
    (
        "item.launch_at_login.desc",
        "Start the app when you sign in.",
        "Khởi động ứng dụng khi bạn đăng nhập.",
        "登录时启动应用。",
        "サインイン時にアプリを起動します。",
    ),
    (
        "item.notifications",
        "Enable Notifications",
        "Bật thông báo",
        "启用通知",
        "通知を有効化",
    ),
    (
        "item.notifications.desc",
        "Allow in-app notifications.",
        "Cho phép thông báo trong ứng dụng.",
        "允许应用内通知。",
        "アプリ内通知を許可します。",
    ),
    (
        "item.test_notification",
        "Test Notification",
        "Thử thông báo",
        "测试通知",
        "テスト通知",
    ),
    (
        "item.test_notification.desc",
        "Push a sample in-app notification.",
        "Gửi một thông báo mẫu trong ứng dụng.",
        "推送一条示例应用内通知。",
        "サンプルのアプリ内通知を送信します。",
    ),
    (
        "item.username",
        "Username",
        "Tên người dùng",
        "用户名",
        "ユーザー名",
    ),
    (
        "item.username.desc",
        "Your unique handle.",
        "Tên định danh duy nhất của bạn.",
        "你的唯一标识。",
        "あなた固有のハンドル名。",
    ),
    (
        "item.display_name",
        "Display Name",
        "Tên hiển thị",
        "显示名称",
        "表示名",
    ),
    (
        "item.display_name.desc",
        "Shown instead of your username.",
        "Hiển thị thay cho tên người dùng.",
        "显示此名称而非用户名。",
        "ユーザー名の代わりに表示されます。",
    ),
    ("item.preview", "Preview", "Xem trước", "预览", "プレビュー"),
    (
        "item.preview.desc",
        "How your profile appears to others.",
        "Hồ sơ của bạn hiển thị với người khác như thế nào.",
        "你的个人资料展示给他人的样子。",
        "他の人に表示されるプロフィールの見え方。",
    ),
    (
        "item.website",
        "Website",
        "Trang web",
        "网站",
        "ウェブサイト",
    ),
    (
        "item.website.desc",
        "Framework website and documentation.",
        "Trang web và tài liệu của framework.",
        "框架网站与文档。",
        "フレームワークのサイトとドキュメント。",
    ),
    ("item.about", "About", "Giới thiệu", "关于", "情報"),
    (
        "item.about.desc",
        "Show the about dialog.",
        "Hiển thị hộp thoại giới thiệu.",
        "显示关于对话框。",
        "情報ダイアログを表示します。",
    ),
    // Actions
    ("action.send", "Send", "Gửi", "发送", "送信"),
    (
        "action.website_label",
        "gpui-kit.com",
        "gpui-kit.com",
        "gpui-kit.com",
        "gpui-kit.com",
    ),
    (
        "action.about_label",
        "About…",
        "Giới thiệu…",
        "关于…",
        "情報…",
    ),
    // Rail
    (
        "rail.light_mode",
        "Light mode",
        "Chế độ sáng",
        "浅色模式",
        "ライトモード",
    ),
    (
        "rail.dark_mode",
        "Dark mode",
        "Chế độ tối",
        "深色模式",
        "ダークモード",
    ),
    // Notifications
    (
        "notify.test_message",
        "Hello from settings-demo!",
        "Xin chào từ settings-demo!",
        "来自 settings-demo 的问候！",
        "settings-demo からこんにちは！",
    ),
    // Dialog
    (
        "dialog.about_title",
        "About settings-demo",
        "Giới thiệu settings-demo",
        "关于 settings-demo",
        "settings-demo について",
    ),
    (
        "dialog.about_body",
        "Built with GPUI Kit — the Rust desktop framework from Longbridge.",
        "Được xây dựng bằng GPUI Kit — framework desktop Rust từ Longbridge.",
        "使用 GPUI Kit 构建 —— 来自 Longbridge 的 Rust 桌面框架。",
        "GPUI Kit で構築 — Longbridge の Rust デスクトップフレームワーク。",
    ),
    // Status chips
    ("status.enabled", "Enabled", "Đã bật", "已启用", "有効"),
    ("status.disabled", "Disabled", "Đã tắt", "已禁用", "無効"),
    (
        "status.version",
        "Version",
        "Phiên bản",
        "版本",
        "バージョン",
    ),
    // About
    (
        "about.tagline",
        "A settings-style demo app built with GPUI Kit.",
        "Ứng dụng demo kiểu cài đặt được xây dựng bằng GPUI Kit.",
        "一个使用 GPUI Kit 构建的设置风格演示应用。",
        "GPUI Kit で構築した設定風デモアプリ。",
    ),
];

/// Translate `key` for locale `lang` (`"en"` | `"vi"` | `"zh-CN"` | `"ja"`).
///
/// Callsite keys are string literals, so `key` is `&'static str` and a miss
/// can return it directly — no allocation, no leak. Falls back to English
/// when the key exists but the locale is unknown.
pub fn tr(key: &'static str, lang: &str) -> &'static str {
    for &(k, en, vi, zh, ja) in STRINGS {
        if k == key {
            return match lang {
                "vi" => vi,
                "zh-CN" => zh,
                "ja" => ja,
                _ => en,
            };
        }
    }
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_key_has_all_languages() {
        for &(key, en, vi, zh, ja) in STRINGS {
            for (lang, value) in [("en", en), ("vi", vi), ("zh-CN", zh), ("ja", ja)] {
                assert!(!value.is_empty(), "empty value for {key} in {lang}");
                assert_eq!(tr(key, lang), value, "wrong value for {key} in {lang}");
            }
        }
    }

    #[test]
    fn unknown_language_falls_back_to_english() {
        for &(key, en, ..) in STRINGS {
            assert_eq!(tr(key, "fr"), en);
            assert_eq!(tr(key, ""), en);
        }
    }

    #[test]
    fn unknown_key_returns_key() {
        assert_eq!(tr("no.such.key", "en"), "no.such.key");
        assert_eq!(tr("no.such.key", "vi"), "no.such.key");
    }
}
