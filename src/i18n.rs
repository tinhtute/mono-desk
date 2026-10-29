//! Zero-dependency translation table for the settings-demo app.
//!
//! Supported locales: `"en"`, `"vi"`, `"zh-CN"`, `"ja"`. Brand and product
//! names (GPUI Kit, gpui-kit.com, settings-demo, Longbridge) are intentionally
//! left untranslated in every locale.

/// One row per UI string: `(key, en, vi, zh-CN, ja)`.
#[allow(dead_code)] // wired by settings-surface T2
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
    (
        "appearance.cycle",
        "Toggle light / dark",
        "Chuyển sáng / tối",
        "切换浅色 / 深色",
        "ライト / ダークを切り替え",
    ),
    (
        "nav.all_settings",
        "All settings",
        "Tất cả cài đặt",
        "所有设置",
        "すべての設定",
    ),
    (
        "nav.preferences",
        "Preferences",
        "Tuỳ chọn",
        "偏好设置",
        "環境設定",
    ),
    (
        "nav.workspace",
        "Workspace",
        "Không gian làm việc",
        "工作区",
        "ワークスペース",
    ),
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
    ("item.appearance", "Appearance", "Giao diện", "外观", "外観"),
    (
        "item.appearance.desc",
        "Always light or dark, or follow the system.",
        "Luôn sáng hoặc tối, hoặc theo hệ thống.",
        "固定浅色或深色，或跟随系统。",
        "常にライト／ダーク、またはシステムに従います。",
    ),
    (
        "preview.sample",
        "The quick brown fox jumps over the lazy dog.",
        "Con cáo nâu nhanh nhẹn nhảy qua con chó lười biếng.",
        "敏捷的棕色狐狸跳过了懒狗。",
        "すばやい茶色のキツネが怠け者の犬を飛び越えます。",
    ),
    ("page.reset", "Reset", "Đặt lại", "重置", "リセット"),
    (
        "group.changed",
        "Changed from defaults",
        "Khác mặc định",
        "已修改",
        "変更あり",
    ),
    (
        "reset.nothing",
        "Nothing has changed",
        "Chưa có gì thay đổi",
        "没有改动",
        "変更はありません",
    ),
    (
        "reset.nothing.desc",
        "Every setting still holds its default value.",
        "Mọi cài đặt vẫn ở giá trị mặc định.",
        "所有设置仍为默认值。",
        "すべての設定が既定値のままです。",
    ),
    (
        "reset.all",
        "Reset everything",
        "Đặt lại tất cả",
        "全部重置",
        "すべてリセット",
    ),
    (
        "reset.all.confirm",
        "Press again to confirm",
        "Nhấn lần nữa để xác nhận",
        "再按一次确认",
        "もう一度押して確定",
    ),
    (
        "reset.all.desc",
        "Asks twice before reverting every setting on this page.",
        "Hỏi hai lần trước khi hoàn tác mọi cài đặt trên trang này.",
        "恢复本页所有设置前会二次确认。",
        "このページの全設定を戻す前に二度確認します。",
    ),
    (
        "reset.theme_default",
        "theme default",
        "mặc định của giao diện",
        "主题默认",
        "テーマ既定",
    ),
    ("state.on", "On", "Bật", "开启", "オン"),
    ("state.off", "Off", "Tắt", "关闭", "オフ"),
    ("appearance.light", "Light", "Sáng", "浅色", "ライト"),
    ("appearance.dark", "Dark", "Tối", "深色", "ダーク"),
    (
        "appearance.system",
        "System",
        "Hệ thống",
        "跟随系统",
        "システム",
    ),
    (
        "item.accent",
        "Accent color",
        "Màu nhấn",
        "强调色",
        "アクセントカラー",
    ),
    (
        "item.accent.desc",
        "Buttons, links and what is chosen.",
        "Nút, liên kết và mục đang chọn.",
        "按钮、链接和选中项的颜色。",
        "ボタン、リンク、選択項目の色。",
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
    // Units
    ("unit.percent", "%", "%", "%", "%"),
    // Background (glass) group
    ("group.background", "Background", "Nền", "背景", "背景"),
    (
        "item.background_translucent",
        "Translucent window",
        "Cửa sổ trong suốt",
        "透明窗口",
        "半透明ウィンドウ",
    ),
    (
        "item.background_translucent.desc",
        "Let the desktop show through the window.",
        "Cho phép nhìn xuyên qua cửa sổ ra màn hình nền.",
        "让桌面透过窗口显示。",
        "デスクトップがウィンドウ越しに見えるようにします。",
    ),
    (
        "item.window_opacity",
        "Window opacity",
        "Độ đục của cửa sổ",
        "窗口不透明度",
        "ウィンドウの不透明度",
    ),
    (
        "item.window_opacity.desc",
        "Alpha of the single frosted window layer, 40–100%. One layer for the \
         whole window; the sidebar tone and content sit on top of it.",
        "Độ trong suốt của lớp kính phủ toàn cửa sổ, 40–100%. Một lớp duy nhất \
         cho cả cửa sổ; thanh bên và nội dung nằm trên lớp đó.",
        "整窗毛玻璃层的透明度，40–100%。整个窗口只有一层；侧边栏色调和内容叠加其上。",
        "ウィンドウ全体のすりガラス層の不透明度（40〜100%）。層は1枚だけで、サイドバーの色調とコンテンツはその上に載ります。",
    ),
    (
        "item.sidebar_tone",
        "Sidebar tone",
        "Độ đậm thanh bên",
        "侧边栏色调",
        "サイドバーの色調",
    ),
    (
        "item.sidebar_tone.desc",
        "Strength of the sidebar's ink tint over the glass, 0–100%. It is not \
         a second surface: 0 leaves only the hairline separator.",
        "Độ đậm lớp phủ màu của thanh bên trên nền kính, 0–100%. Đây không phải \
         bề mặt thứ hai: 0 chỉ còn đường viền mảnh.",
        "侧边栏在玻璃上的墨水色调强度，0–100%。它不是第二层表面：0 时仅剩细分割线。",
        "ガラス上のサイドバーのインク色の濃さ（0〜100%）。第2の面ではなく、0では細い区切り線だけが残ります。",
    ),
    (
        "item.background_blur",
        "Background blur",
        "Làm mờ nền",
        "背景模糊",
        "背景のぼかし",
    ),
    (
        "item.background_blur.desc",
        "Ask the system to frost what is behind the window. macOS vibrancy; \
         compositor-dependent on Linux, experimental on Windows.",
        "Yêu cầu hệ thống làm mờ nội dung phía sau cửa sổ. Trên macOS là vibrancy; \
         trên Linux phụ thuộc compositor, trên Windows còn thử nghiệm.",
        "请求系统对窗口背后的内容做磨砂处理。macOS 使用 vibrancy；Linux 取决于合成器，Windows 仍属实验特性。",
        "ウィンドウ背後の内容をぼかすよう OS に要求します。macOS は vibrancy、Linux はコンポジタ依存、Windows は実験的です。",
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
    // Sidebar search (`settings-surface` T4).
    (
        "settings.clear",
        "Clear search",
        "Xoá tìm kiếm",
        "清除搜索",
        "検索をクリア",
    ),
    (
        "settings.empty",
        "No sections match",
        "Không có mục nào khớp",
        "没有匹配的设置",
        "一致するセクションがありません",
    ),
    // Per-row reset action on the Reset page (`settings-surface` T2).
    ("action.reset", "Reset", "Đặt lại", "重置", "リセット"),
    (
        "settings.search",
        "Search settings",
        "Tìm cài đặt",
        "搜索设置",
        "設定を検索",
    ),
    // Appearance page (`settings-surface` T3, zeron's custom page structure).
    ("page.appearance", "Appearance", "Giao diện", "外观", "外観"),
    (
        "appearance.scheme",
        "Color scheme",
        "Bảng màu",
        "配色方案",
        "カラースキーム",
    ),
    (
        "appearance.material",
        "Material and background",
        "Chất liệu và nền",
        "材质与背景",
        "マテリアルと背景",
    ),
    (
        "appearance.motion",
        "Motion",
        "Chuyển động",
        "动效",
        "モーション",
    ),
    (
        "appearance.fonts",
        "Fonts and layout",
        "Phông chữ và bố cục",
        "字体与布局",
        "フォントとレイアウト",
    ),
    (
        "appearance.light_theme",
        "Light theme",
        "Giao diện sáng",
        "浅色主题",
        "ライトテーマ",
    ),
    (
        "appearance.dark_theme",
        "Dark theme",
        "Giao diện tối",
        "深色主题",
        "ダークテーマ",
    ),
    (
        "appearance.accent",
        "Accent color",
        "Màu nhấn",
        "强调色",
        "アクセントカラー",
    ),
    (
        "appearance.accent.desc",
        "The interactive accent used for selections and focus.",
        "Màu nhấn cho lựa chọn và tiêu điểm.",
        "用于选择与焦点的强调色。",
        "選択とフォーカスに使うアクセントカラー。",
    ),
    ("appearance.glass", "Glass", "Kính", "玻璃", "ガラス"),
    (
        "appearance.glass.desc",
        "How the window surface is drawn.",
        "Cách vẽ bề mặt cửa sổ.",
        "窗口表面的绘制方式。",
        "ウィンドウ表面の描画方法。",
    ),
    (
        "appearance.surface.default",
        "Theme default",
        "Theo giao diện",
        "跟随主题",
        "テーマに従う",
    ),
    (
        "appearance.surface.frosted",
        "Translucent surfaces",
        "Bề mặt trong mờ",
        "半透明表面",
        "半透明サーフェス",
    ),
    (
        "appearance.surface.opaque",
        "Solid surfaces",
        "Bề mặt đặc",
        "不透明表面",
        "不透明サーフェス",
    ),
    (
        "appearance.motion.reduce",
        "Reduce motion",
        "Giảm chuyển động",
        "减少动效",
        "モーションを減らす",
    ),
    (
        "appearance.motion.reduce.desc",
        "Skip the hover and selection fades.",
        "Bỏ hiệu ứng mờ khi rê chuột và chọn.",
        "跳过悬停与选择的渐隐动效。",
        "ホバーと選択のフェードを省略します。",
    ),
    (
        "notifications.test_sent",
        "Test notification sent.",
        "Đã gửi thông báo thử.",
        "已发送测试通知。",
        "テスト通知を送信しました。",
    ),
];

/// Translate `key` for locale `lang` (`"en"` | `"vi"` | `"zh-CN"` | `"ja"`).
///
/// Callsite keys are string literals, so `key` is `&'static str` and a miss
/// can return it directly — no allocation, no leak. Falls back to English
/// when the key exists but the locale is unknown.
#[allow(dead_code)] // wired by settings-surface T2
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
