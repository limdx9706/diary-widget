# Diary Widget 美观性与功能性改进设计

## 背景

Diary Widget 是一个基于 Tauri 2 + React 的 Windows 桌面日记小组件，功能上基本可用但存在以下核心问题：

1. **窗口控制按钮缺陷**：最小化/关闭按钮使用纯文本字符 `-` 和 `x`，无 hover 效果，视觉上不专业。同时，Tauri capability 配置缺少 `core:window:*` 权限，导致按钮操作可能静默失败。
2. **UI 简陋**：整体样式过于基础，缺少过渡动画、加载状态指示、滚动条美化等现代 UI 特性。
3. **配置局限**：配置路径硬编码 `%APPDATA%` (仅 Windows)，窗口缺少最小尺寸限制导致过小时 UI 崩坏。
4. **代码质量**：存在未使用的依赖 (`gray_matter`, `yaml-rust2`, `anyhow`)，Toast 定时器使用函数属性（非标准模式），写入成功后不清空表单。

## 改进方案

### 1. 窗口控制按钮修复

**变更文件**: `src/App.jsx`, `src/App.css`, `src-tauri/capabilities/default.json`

- 替换文本字符为 SVG 图标（最小化: 横线图标, 关闭: X 图标）
- 添加 hover 状态：最小化按钮悬停变亮，关闭按钮悬停变红
- 在 capabilities 中添加 `core:window:default`, `core:window:allow-close`, `core:window:allow-minimize`, `core:window:allow-start-dragging`, `core:window:allow-toggle-maximize` 权限
- 标题栏支持双击切换最大化

### 2. UI 美化

**变更文件**: `src/App.css`, `src/App.jsx`, `src/components/Toast.jsx`

- **CSS 变量系统**: 引入设计 token（颜色、圆角、过渡），确保一致性和可维护性
- **深色毛玻璃主题**: 使用 `backdrop-filter: blur()` 和半透明背景实现现代 glassmorphism 风格
- **标题栏**: 增加应用图标 SVG、调整间距、窗口控制按钮样式化
- **表单**: 聚焦时发光边框 (box-shadow glow)、占位文本改进、日期/时间控件 dark scheme
- **按钮**: 主按钮带阴影和微交互（hover 上浮 1px）、添加加载旋转动画
- **Toast**: 成功/错误用半透明背景 + 对应颜色文字（而非纯色背景白字），添加入场上滑动画和 SVG 图标
- **滚动条**: 使用 `scrollbar-width: thin` 和 webkit 自定义样式
- **设置页**: 居中布局、添加锁图标、`code` 标签高亮

### 3. Tauri 配置完善

**变更文件**: `src-tauri/tauri.conf.json`

- 添加 `minWidth: 320`, `minHeight: 400` 防止窗口过小
- 将默认窗口高度从 480 调整为 520 以适应新布局

### 4. Rust 后端优化

**变更文件**: `src-tauri/Cargo.toml`, `src-tauri/src/main.rs`

- **移除未使用依赖**: `gray_matter`, `yaml-rust2`, `anyhow`（代码中均未引用）
- **添加 `dirs` crate**: 实现跨平台配置目录发现（Windows 仍优先使用 APPDATA，其他系统用 XDG 标准路径）
- **改善 `should_append_without_sorting`**: 使用 `map_or` 替代 `is_empty()` + `last().expect()` 的分离检查
- **字符串构造**: `"xxx".to_string()` 简化为 `"xxx".into()` 在 serde_yaml Value 构造处

### 5. 前端代码质量

**变更文件**: `src/App.jsx`

- **Toast 定时器**: 从函数属性 (`showToast.timerId`) 改为 `useRef`，避免非标准用法和潜在内存泄漏，组件卸载时清理
- **写入后清空**: `onWrite` 成功后清空 `title` 和 `content`，刷新日期和时间
- **`showToast` 稳定引用**: 使用 `useCallback` 包装避免依赖重建
- **`appWindow` 缓存**: 将 `getCurrentWindow()` 调用提升到模块级别，避免每次渲染/事件重复调用

## 所有变更文件清单

| 文件 | 变更类型 |
|------|---------|
| `src/App.jsx` | 重写 |
| `src/App.css` | 重写 |
| `src/components/Toast.jsx` | 改进 |
| `src-tauri/src/main.rs` | 优化 |
| `src-tauri/Cargo.toml` | 依赖清理 |
| `src-tauri/tauri.conf.json` | 配置增强 |
| `src-tauri/capabilities/default.json` | 权限补全 |
