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
    ("group.channels", "Channels", "Kênh", "渠道", "チャンネル"),
    (
        "group.digest",
        "Digest",
        "Bản tin tổng hợp",
        "摘要",
        "ダイジェスト",
    ),
    ("group.region", "Region", "Khu vực", "地区", "地域"),
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
        "item.username_placeholder",
        "your-handle",
        "tên-của-bạn",
        "你的用户名",
        "your-handle",
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
    (
        "item.channel_all",
        "All channels",
        "Tất cả kênh",
        "全部渠道",
        "すべてのチャンネル",
    ),
    ("item.channel_email", "Email", "Email", "电子邮件", "メール"),
    (
        "item.channel_desktop",
        "Desktop",
        "Trên máy tính",
        "桌面",
        "デスクトップ",
    ),
    (
        "item.channel_digest",
        "Digest",
        "Bản tin tổng hợp",
        "摘要",
        "ダイジェスト",
    ),
    (
        "item.channels.desc",
        "Where notifications are delivered.",
        "Nơi nhận thông báo.",
        "通知的投递渠道。",
        "通知の配信先。",
    ),
    (
        "item.digest.desc",
        "How often digest notifications arrive.",
        "Tần suất nhận bản tin tổng hợp.",
        "摘要通知的发送频率。",
        "ダイジェスト通知の頻度。",
    ),
    (
        "digest.immediate",
        "Immediate",
        "Ngay lập tức",
        "即时",
        "即時",
    ),
    ("digest.daily", "Daily", "Hằng ngày", "每日", "毎日"),
    ("digest.weekly", "Weekly", "Hằng tuần", "每周", "毎週"),
    (
        "item.timezone",
        "Time Zone",
        "Múi giờ",
        "时区",
        "タイムゾーン",
    ),
    (
        "item.timezone.desc",
        "Used for timestamps across the app.",
        "Dùng cho nhãn thời gian trong ứng dụng.",
        "用于应用内的时间戳。",
        "アプリ内のタイムスタンプに使用します。",
    ),
    (
        "tz.placeholder",
        "Pick a time zone",
        "Chọn múi giờ",
        "选择时区",
        "タイムゾーンを選択",
    ),
    (
        "tz.search_placeholder",
        "Search cities…",
        "Tìm thành phố…",
        "搜索城市…",
        "都市を検索…",
    ),
    (
        "tz.system",
        "Follow system time zone",
        "Theo múi giờ hệ thống",
        "跟随系统时区",
        "システムのタイムゾーンに従う",
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
    // Field validation
    (
        "field.username_too_short",
        "Use at least 3 characters.",
        "Dùng ít nhất 3 ký tự.",
        "至少使用 3 个字符。",
        "3文字以上で入力してください。",
    ),
    (
        "field.username_too_long",
        "Use at most 16 characters.",
        "Dùng tối đa 16 ký tự.",
        "最多使用 16 个字符。",
        "16文字以内で入力してください。",
    ),
    (
        "field.username_invalid",
        "Only lowercase letters, digits, - and _.",
        "Chỉ dùng chữ thường, số, - và _.",
        "仅限小写字母、数字、- 和 _。",
        "小文字・数字・-・_ のみ使用できます。",
    ),
    (
        "field.username_ok",
        "Looks good.",
        "Hợp lệ.",
        "看起来不错。",
        "問題ありません。",
    ),
    // Finance UI showcase (cloned from the yurygok reference)
    (
        "page.finance",
        "Finance UI",
        "Giao diện tài chính",
        "财务界面",
        "ファイナンスUI",
    ),
    (
        "page.finance.desc",
        "Components cloned from x.com/yurygok/status/2105191658337841531 — sample data is the reference's own.",
        "Các component được clone từ x.com/yurygok/status/2105191658337841531 — dữ liệu mẫu giữ nguyên từ bản gốc.",
        "从 x.com/yurygok/status/2105191658337841531 克隆的组件 —— 示例数据保留原样。",
        "x.com/yurygok/status/2105191658337841531 からクローンしたコンポーネント — サンプルデータは参照のまま。",
    ),
    // About
    (
        "about.tagline",
        "A settings-style demo app built with GPUI Kit.",
        "Ứng dụng demo kiểu cài đặt được xây dựng bằng GPUI Kit.",
        "一个使用 GPUI Kit 构建的设置风格演示应用。",
        "GPUI Kit で構築した設定風デモアプリ。",
    ),
    ("about.tab.info", "Info", "Thông tin", "信息", "情報"),
    (
        "about.tab.licenses",
        "Licenses",
        "Giấy phép",
        "许可证",
        "ライセンス",
    ),
    (
        "about.tab.changelog",
        "Changelog",
        "Nhật ký thay đổi",
        "更新日志",
        "変更履歴",
    ),
    (
        "action.copy_version",
        "Copy version",
        "Sao chép phiên bản",
        "复制版本",
        "バージョンをコピー",
    ),
    (
        "about.info.body",
        "A single-binary demo of the GPUI Kit settings components. Pages, fields and notifications mirror a real settings app.",
        "Bản demo một file nhị phân cho các component cài đặt của GPUI Kit. Các trang, trường và thông báo mô phỏng một ứng dụng cài đặt thật.",
        "GPUI Kit 设置组件的单二进制演示。页面、字段与通知均仿照真实设置应用。",
        "GPUI Kit の設定コンポーネントを使った単一バイナリのデモ。ページ・フィールド・通知は実際の設定アプリを再現。",
    ),
    (
        "about.licenses.0",
        "GPUI Kit — Apache-2.0",
        "GPUI Kit — Apache-2.0",
        "GPUI Kit — Apache-2.0",
        "GPUI Kit — Apache-2.0",
    ),
    (
        "about.licenses.1",
        "Lucide icons — ISC",
        "Biểu tượng Lucide — ISC",
        "Lucide 图标 — ISC",
        "Lucide アイコン — ISC",
    ),
    (
        "about.licenses.2",
        "Xanh Mono font — OFL-1.1",
        "Phông chữ Xanh Mono — OFL-1.1",
        "Xanh Mono 字体 — OFL-1.1",
        "Xanh Mono フォント — OFL-1.1",
    ),
    (
        "about.changelog.0",
        "0.1.0 — First settings-demo build: shell, pages, and controls.",
        "0.1.0 — Bản đầu của settings-demo: khung, trang và điều khiển.",
        "0.1.0 — settings-demo 首个版本：外壳、页面与控件。",
        "0.1.0 — settings-demo 初回ビルド：シェル、ページ、コントロール。",
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
