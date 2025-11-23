# 全局快捷键测试指南

## 已注册的全局快捷键

本应用已实现以下全局快捷键:

### 1. Ctrl+Shift+N - 新建任务
- **功能**: 显示窗口并自动导航到任务看板页面
- **使用场景**: 快速打开应用并准备创建新任务
- **测试步骤**:
  1. 最小化或隐藏 DevAssistant 窗口
  2. 在任何地方按 `Ctrl+Shift+N`
  3. 验证窗口显示并聚焦
  4. 验证自动导航到任务看板页面

### 2. Ctrl+Shift+S - SQL历史
- **功能**: 显示窗口并自动导航到SQL历史页面
- **使用场景**: 快速查看或搜索SQL历史记录
- **测试步骤**:
  1. 最小化或隐藏 DevAssistant 窗口
  2. 在任何地方按 `Ctrl+Shift+S`
  3. 验证窗口显示并聚焦
  4. 验证自动导航到SQL历史页面

## 技术实现

### 后端实现 (Rust/Tauri)

1. **依赖添加**: 在 `Cargo.toml` 中添加了 `tauri-plugin-global-shortcut = "2.0"`

2. **插件初始化**: 在 `main.rs` 中初始化全局快捷键插件
   ```rust
   .plugin(tauri_plugin_global_shortcut::Builder::new().build())
   ```

3. **快捷键注册**: 在 `setup()` 钩子中注册快捷键
   ```rust
   app.global_shortcut().on_shortcut("Ctrl+Shift+N", move |app, _shortcut, event| {
       if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
           if let Some(window) = app.get_webview_window("main") {
               let _ = window.show();
               let _ = window.set_focus();
               let _ = window.emit("navigate-to", "/task-board");
           }
       }
   })
   ```

4. **窗口命令**: 创建了 `window_commands.rs` 模块,提供以下命令:
   - `show_window`: 显示窗口并聚焦
   - `hide_window`: 隐藏窗口
   - `toggle_window`: 切换窗口显示状态
   - `show_window_with_route`: 显示窗口并导航到指定路由

### 前端实现 (Vue/TypeScript)

1. **事件监听**: 在 `main.ts` 中添加导航事件监听
   ```typescript
   import { listen } from '@tauri-apps/api/event'

   listen<string>('navigate-to', (event) => {
     const route = event.payload
     router.push(route)
   })
   ```

2. **API 封装**: 创建了 `windowApi.ts` 提供窗口操作API
   ```typescript
   export async function showWindow(): Promise<void>
   export async function hideWindow(): Promise<void>
   export async function toggleWindow(): Promise<void>
   export async function showWindowWithRoute(route: string): Promise<void>
   ```

## 配置文件修改

### tauri.conf.json
- 添加了窗口 label: `"label": "main"`
- 这确保我们可以通过 `get_webview_window("main")` 访问窗口

## 注意事项

1. **快捷键冲突**: 确保 `Ctrl+Shift+N` 和 `Ctrl+Shift+S` 不与系统或其他应用的快捷键冲突

2. **权限要求**:
   - Windows: 可能需要管理员权限来注册某些全局快捷键
   - macOS: 需要在系统偏好设置中授予辅助功能权限
   - Linux: 通常不需要额外权限

3. **跨平台兼容性**: 快捷键在 Windows、macOS 和 Linux 上都能工作,但建议在所有平台上进行测试

4. **应用未启动时**: 全局快捷键只在应用运行时有效。如果应用未启动,快捷键不会有任何效果

## 扩展建议

未来可以考虑添加以下功能:

1. **自定义快捷键**: 允许用户在设置中自定义快捷键
2. **更多快捷键**:
   - `Ctrl+Shift+L`: 工作日志
   - `Ctrl+Shift+H`: 隐藏/显示窗口
3. **快捷键提示**: 在界面上显示可用的快捷键列表
4. **禁用快捷键**: 在设置中提供禁用全局快捷键的选项
