# DevAssistant 更新服务器部署检查清单

## 部署前准备

### 1. 服务器环境检查

- [ ] PHP 版本 >= 5.4
- [ ] Web 服务器（Apache/Nginx）已配置
- [ ] HTTPS 证书已安装（强烈推荐）
- [ ] 目录读写权限正确

### 2. 文件准备

- [ ] `update.php` - 主接口文件
- [ ] `version.json` - 版本配置文件
- [ ] `.htaccess` - Apache 配置（可选，仅 Apache）

### 3. 配置修改

在 `version.json` 中修改以下内容：

- [ ] 版本号（version）
- [ ] 更新说明（notes）
- [ ] 发布日期（pub_date）
- [ ] 安装包 URL（platforms.*.url）
- [ ] 安装包签名（platforms.*.signature）

## 部署步骤

### 步骤 1：上传文件

上传以下文件到服务器：

```
your-server/
├── server/
│   ├── update.php
│   ├── version.json
│   └── .htaccess (Apache)
└── releases/
    └── DevAssistant_x.x.x_x64-setup.nsis.zip
```

### 步骤 2：设置权限

```bash
# 设置文件权限
chmod 644 update.php version.json
chmod 755 server/

# 如果启用日志功能
chmod 666 update.log
```

### 步骤 3：配置 Web 服务器

#### Apache

确保启用以下模块：

```bash
a2enmod headers
a2enmod deflate
a2enmod expires
service apache2 restart
```

#### Nginx

添加以下配置到 nginx.conf：

```nginx
location /server/ {
    add_header Access-Control-Allow-Origin *;
    add_header Access-Control-Allow-Methods "GET, OPTIONS";
    add_header Access-Control-Allow-Headers "Content-Type";

    # 禁止缓存
    add_header Cache-Control "no-cache, no-store, must-revalidate";

    # PHP 配置
    location ~ \.php$ {
        include fastcgi_params;
        fastcgi_pass unix:/var/run/php/php7.4-fpm.sock;
        fastcgi_param SCRIPT_FILENAME $document_root$fastcgi_script_name;
    }
}
```

### 步骤 4：测试接口

使用 curl 测试：

```bash
# 测试有更新的情况
curl "https://your-server.com/server/update.php?target=windows-x86_64&current_version=0.1.0"

# 测试无更新的情况
curl -i "https://your-server.com/server/update.php?target=windows-x86_64&current_version=0.2.0"
```

或使用 `test_update.html` 在浏览器中测试。

### 步骤 5：配置 Tauri 应用

在 `src-tauri/tauri.conf.json` 中配置：

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

## 发布新版本流程

### 1. 构建应用

```bash
npm run tauri build
```

### 2. 上传安装包

上传生成的文件到服务器 releases 目录：

- `src-tauri/target/release/bundle/nsis/DevAssistant_x.x.x_x64-setup.nsis.zip`
- `src-tauri/target/release/bundle/nsis/DevAssistant_x.x.x_x64-setup.nsis.zip.sig`

### 3. 获取签名

```bash
# 读取签名文件内容
cat DevAssistant_x.x.x_x64-setup.nsis.zip.sig
```

### 4. 更新 version.json

```json
{
  "version": "x.x.x",
  "notes": "## 更新内容 vx.x.x\n\n### 新增功能\n- ...\n\n### 修复问题\n- ...",
  "pub_date": "2025-12-06T12:00:00Z",
  "platforms": {
    "windows-x86_64": {
      "signature": "从 .sig 文件复制的内容",
      "url": "https://your-server.com/releases/DevAssistant_x.x.x_x64-setup.nsis.zip"
    }
  }
}
```

### 5. 验证更新

- [ ] 使用 curl 测试接口
- [ ] 使用 test_update.html 测试
- [ ] 在应用中测试自动更新功能

## 安全检查

- [ ] 使用 HTTPS 协议
- [ ] 配置了正确的 CORS 头
- [ ] 签名验证正常工作
- [ ] 禁止直接访问敏感文件（.sig, .log）
- [ ] 定期更新服务器和 PHP 版本

## 监控和维护

### 日志监控

如果启用了日志功能，定期检查：

```bash
tail -f update.log
```

### 磁盘空间

定期清理旧版本安装包：

```bash
# 保留最近 3 个版本
ls -t releases/ | tail -n +4 | xargs -I {} rm releases/{}
```

### 备份

定期备份重要文件：

```bash
# 备份脚本示例
#!/bin/bash
DATE=$(date +%Y%m%d)
tar -czf backup_${DATE}.tar.gz server/ releases/
```

## 故障排查

### 问题 1：接口返回 404

**可能原因：**
- URL 路径错误
- Web 服务器配置问题

**解决方法：**
```bash
# 检查文件是否存在
ls -la /path/to/server/update.php

# 检查 Web 服务器配置
# Apache: httpd -S
# Nginx: nginx -t
```

### 问题 2：CORS 错误

**可能原因：**
- 缺少 CORS 头
- Web 服务器配置错误

**解决方法：**
- 检查 .htaccess 或 nginx 配置
- 确保启用了 mod_headers (Apache)

### 问题 3：签名验证失败

**可能原因：**
- 公钥配置错误
- 签名内容复制错误
- 安装包被篡改

**解决方法：**
- 重新生成密钥对
- 确保签名完整复制
- 重新构建和签名安装包

## 联系支持

如有问题，请：

1. 查看服务器日志
2. 使用 test_update.html 测试
3. 检查本文档的故障排查部分

## 版本历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0.0 | 2025-12-06 | 初始版本 |
