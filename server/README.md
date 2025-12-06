# DevAssistant 自动更新服务器

这个目录包含了 DevAssistant 应用的自动更新服务器端文件。

## 文件说明

- `update.php` - 主要的更新接口，处理客户端的更新请求
- `version.json` - 版本配置文件，存储最新版本信息
- `README.md` - 本说明文档

## 快速开始

### 1. 部署到服务器

将 `update.php` 和 `version.json` 上传到您的 Web 服务器。

**服务器要求：**
- PHP 5.4 或更高版本
- 支持 HTTPS（强烈推荐）
- 可读写权限（如需启用日志功能）

**示例部署路径：**
```
https://your-server.com/
├── server/
│   ├── update.php
│   └── version.json
```

### 2. 配置 Tauri 应用

在 `src-tauri/tauri.conf.json` 中配置更新服务器地址：

```json
{
  "tauri": {
    "updater": {
      "active": true,
      "endpoints": [
        "https://your-server.com/server/update.php"
      ],
      "dialog": true,
      "pubkey": "YOUR_PUBLIC_KEY_HERE"
    }
  }
}
```

### 3. 发布新版本

每次发布新版本时：

1. **构建应用并生成签名**
   ```bash
   npm run tauri build
   ```

2. **上传安装包到服务器**

   上传以下文件到您的服务器：
   - `DevAssistant_x.x.x_x64-setup.nsis.zip`
   - `DevAssistant_x.x.x_x64-setup.nsis.zip.sig`

3. **更新 version.json**

   编辑 `version.json` 文件：
   ```json
   {
     "version": "0.2.0",
     "notes": "## 更新内容\n- 新功能说明\n- 修复内容",
     "pub_date": "2025-12-06T12:00:00Z",
     "platforms": {
       "windows-x86_64": {
         "signature": "从 .sig 文件复制的签名内容",
         "url": "https://your-server.com/releases/DevAssistant_0.2.0_x64-setup.nsis.zip"
       }
     }
   }
   ```

## API 说明

### 请求示例

```
GET https://your-server.com/server/update.php?target=windows-x86_64&current_version=0.1.0
```

### 参数说明

| 参数 | 类型 | 必需 | 说明 |
|------|------|------|------|
| target | string | 是 | 平台标识，如 `windows-x86_64` |
| current_version | string | 是 | 当前版本号，如 `0.1.0` |

### 支持的平台标识

- `windows-x86_64` - Windows 64位
- `windows-i686` - Windows 32位
- `darwin-x86_64` - macOS Intel
- `darwin-aarch64` - macOS Apple Silicon
- `linux-x86_64` - Linux 64位

### 响应说明

#### 有新版本可用（200）

```json
{
  "version": "0.2.0",
  "notes": "## 更新内容\n- 新功能\n- 修复问题",
  "pub_date": "2025-12-06T12:00:00Z",
  "platforms": {
    "windows-x86_64": {
      "signature": "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkKUldUTE...",
      "url": "https://your-server.com/releases/DevAssistant_0.2.0_x64-setup.nsis.zip"
    }
  }
}
```

#### 已是最新版本（204）

无响应内容，状态码为 204 No Content。

#### 参数错误（400）

```json
{
  "error": "Missing required parameter: target"
}
```

#### 平台不支持（404）

```json
{
  "error": "No update available for platform: unknown-platform"
}
```

#### 服务器错误（500）

```json
{
  "error": "Internal server error: ..."
}
```

## version.json 配置说明

```json
{
  "version": "0.2.0",              // 最新版本号
  "notes": "## 更新内容\n...",     // 更新说明（支持 Markdown）
  "pub_date": "2025-12-06T12:00:00Z",  // 发布日期（ISO 8601 格式）
  "platforms": {                    // 各平台的更新信息
    "windows-x86_64": {
      "signature": "...",           // 安装包签名（从 .sig 文件获取）
      "url": "https://..."          // 安装包下载地址
    }
  }
}
```

## 版本签名

Tauri 使用签名来验证更新包的完整性和安全性。

### 生成密钥对

```bash
npm run tauri signer generate -- -w ~/.tauri/myapp.key
```

这将生成：
- 私钥文件（保密，用于签名）
- 公钥（配置到 tauri.conf.json）

### 获取签名

构建应用后，在 `src-tauri/target/release/bundle/nsis/` 目录下会生成 `.sig` 文件，内容即为签名。

## 日志功能

默认情况下，日志功能是关闭的。如需启用：

1. 打开 `update.php`
2. 找到 `logMessage` 函数
3. 取消注释 `file_put_contents` 这行代码
4. 确保服务器有写入权限

日志将保存在 `update.log` 文件中。

## 安全建议

1. **使用 HTTPS**：强烈建议使用 HTTPS 协议，防止中间人攻击
2. **验证签名**：确保在 tauri.conf.json 中配置了正确的公钥
3. **限制访问**：可以配置服务器只允许特定 IP 或域名访问
4. **定期备份**：定期备份 version.json 和安装包文件

## 测试

### 本地测试

可以使用 curl 或浏览器测试接口：

```bash
# 测试有新版本的情况
curl "http://localhost/server/update.php?target=windows-x86_64&current_version=0.1.0"

# 测试已是最新版本的情况
curl -i "http://localhost/server/update.php?target=windows-x86_64&current_version=0.2.0"
```

### 应用内测试

在开发环境中，可以修改 `src-tauri/tauri.conf.json` 中的版本号为较低版本，然后运行应用测试更新功能。

## 常见问题

### Q: 更新检查失败怎么办？

A: 检查以下几点：
1. 服务器 URL 是否正确
2. version.json 格式是否正确
3. 服务器是否可访问
4. 是否配置了正确的 CORS 头

### Q: 如何支持多个平台？

A: 在 version.json 中为每个平台添加对应的配置项，并确保上传了对应平台的安装包。

### Q: 签名验证失败怎么办？

A: 确保：
1. tauri.conf.json 中的公钥正确
2. version.json 中的签名来自对应安装包的 .sig 文件
3. 安装包未被篡改

## 维护

- 定期清理旧版本的安装包以节省存储空间
- 定期检查日志文件大小（如果启用了日志）
- 确保服务器有足够的带宽处理更新下载

## 许可证

本项目遵循 MIT 许可证。
