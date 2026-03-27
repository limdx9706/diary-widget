# Diary Widget 美观性与功能性改进设计

## 1. 发现的问题

### 1.1 关键 Bug

| 问题 | 严重程度 | 描述 |
|------|----------|------|
| 窗口权限缺失 | **严重** | `capabilities/default.json` 未包含 `core:window:default`、`core:window:allow-minimize`、`core:window:allow-close`、`core:window:allow-start-dragging`，导致自定义标题栏的最小化/关闭/拖拽可能失效 |
| skipTaskbar + minimize 冲突 | **严重** | `skipTaskbar: true` 时最小化窗口后无法通过任务栏恢复，用户丢失窗口 |
| 最小化按钮不可靠 | **高** | 缺少窗口权限声明，`getCurrentWindow().minimize()` 在 Tauri 2 中会被安全策略拦截 |

### 1.2 Rust 代码问题

| 问题 | 描述 |
|------|------|
| 未使用的依赖 | `gray_matter`、`yaml-rust2`、`anyhow` 在代码中未被使用，增加编译时间和二进制体积 |
| 正则表达式重复编译 | 每次调用 `shift_headings_down_one_if_has_h1`/`parse_diary_body_entries`/`write_diary_to_file` 都重新编译正则 |
| 单文件架构 | 所有逻辑集中在 `main.rs`（509 行），不利于维护 |
| 错误处理 | 使用 `String` 作为错误类型，未利用 `anyhow` 或自定义错误枚举 |
| 平台硬编码 | `APPDATA` 环境变量仅 Windows 有效，不可跨平台 |

### 1.3 UI/UX 问题

| 问题 | 描述 |
|------|------|
| 标题栏按钮简陋 | 使用纯文本 `-` 和 `x`，无图标、无悬停效果 |
| 按钮无视觉反馈 | 通用按钮样式，无 hover/active 状态 |
| Toast 无动画 | 突然出现/消失，体验生硬 |
| 表单控件粗糙 | 输入框无聚焦样式、无过渡动画 |
| 无滚动条美化 | 长内容时默认滚动条破坏美观 |
| 字符计数缺失 | 用户不知道内容长度 |

## 2. 改进方案

### 2.1 修复权限配置

在 `capabilities/default.json` 中添加窗口操作权限。

### 2.2 解决 skipTaskbar 问题

改用 `hide`/`show` 替代 `minimize`，或移除 `skipTaskbar` 以允许正常最小化恢复。

### 2.3 Rust 代码优化

- 移除未使用的 crate (`gray_matter`, `yaml-rust2`, `anyhow`)
- 使用 `std::sync::LazyLock` 缓存正则表达式
- 将代码拆分为模块：`config.rs`、`diary.rs`、`commands.rs`
- 改进错误处理

### 2.4 UI 精美化

- 标题栏按钮使用 SVG 图标 + 悬停效果
- 关闭按钮悬停变红
- 输入框聚焦发光效果
- Toast 添加滑入/滑出动画
- 美化滚动条
- 添加字符计数器
- 提交按钮渐变色 + 涟漪效果
