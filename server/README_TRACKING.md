# DevAssistant 用户追踪系统部署指南

## 功能概述

本系统用于追踪 DevAssistant 软件的使用情况，包括：
- 设备信息收集（MAC地址、计算机名、用户名等）
- 定期心跳上报（每2小时）
- 设备禁用/解禁管理
- 用户申诉功能

## 目录结构

```
server/
├── config/
│   └── database.php          # 数据库配置文件（需要修改）
├── sql/
│   └── init.sql              # 数据库初始化脚本
├── includes/
│   ├── common.php            # 公共函数库
│   └── auth.php              # 管理员认证模块
├── api/
│   ├── heartbeat.php         # 心跳上报接口
│   ├── check_status.php      # 状态检查接口
│   ├── appeal.php            # 申诉提交接口
│   └── appeal_status.php     # 申诉状态查询接口
└── admin/
    ├── login.php             # 管理员登录页面
    ├── logout.php            # 管理员退出
    ├── index.php             # 管理后台首页
    ├── devices.php           # 设备管理页面
    ├── device_detail.php     # 设备详情页面
    ├── appeals.php           # 申诉管理页面
    ├── logs.php              # 操作日志页面
    └── api.php               # 管理后台 API
```

## 部署步骤

### 1. 配置数据库

编辑 `config/database.php`，填入您的数据库信息：

```php
return [
    'host' => 'localhost',
    'port' => 3306,
    'database' => 'dev_assistant',
    'username' => 'your_username',
    'password' => 'your_password',
    'charset' => 'utf8mb4',
    'prefix' => 'da_',
];
```

### 2. 创建数据库

```sql
CREATE DATABASE dev_assistant CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
```

### 3. 执行初始化脚本

```bash
mysql -u your_username -p dev_assistant < sql/init.sql
```

### 4. 上传文件到服务器

将以下目录上传到您的 Web 服务器：
- `config/`
- `sql/`（可选，仅用于备份）
- `includes/`
- `api/`
- `admin/`

### 5. 配置权限

```bash
# 确保日志目录可写
chmod 755 logs/

# 确保配置文件安全
chmod 644 config/database.php
```

### 6. 访问管理后台

访问 `https://your-domain.com/server/admin/login.php`

默认管理员账号：
- 用户名：`admin`
- 密码：`admin123`

**重要：首次登录后请立即修改密码！**

## API 接口说明

### 心跳上报
- **URL**: `POST /api/heartbeat.php`
- **参数** (JSON):
  ```json
  {
    "mac_address": "AA:BB:CC:DD:EE:FF",
    "hostname": "PC-NAME",
    "username": "user",
    "os_info": "Windows 10",
    "app_version": "1.0.0",
    "local_ip": "192.168.1.100"
  }
  ```
- **响应**:
  - 200: 上报成功
  - 403: 设备已被禁用

### 状态检查
- **URL**: `POST /api/check_status.php`
- **参数**: `{ "mac_address": "..." }`
- **响应**:
  - 200: 状态正常
  - 403: 设备已被禁用

### 提交申诉
- **URL**: `POST /api/appeal.php`
- **参数**:
  ```json
  {
    "mac_address": "...",
    "contact": "email@example.com",
    "reason": "申诉理由..."
  }
  ```

### 查询申诉状态
- **URL**: `POST /api/appeal_status.php`
- **参数**: `{ "mac_address": "..." }`

## 客户端配置

客户端已集成设备追踪功能，会在启动后自动：
1. 收集设备信息
2. 检查禁用状态
3. 每2小时上报心跳

如需修改服务器地址，编辑 `src/services/deviceTracker.ts`:

```typescript
const API_BASE_URL = 'http://your-domain.com/server/api'
```

## 安全建议

1. 使用 HTTPS 确保数据传输安全
2. 定期更换管理员密码
3. 限制管理后台的访问 IP
4. 定期备份数据库
5. 监控异常访问日志

## 常见问题

### Q: 如何修改管理员密码？

直接在数据库中更新：

```sql
UPDATE da_admins
SET password = '$2y$10$...(password_hash生成的密码)'
WHERE username = 'admin';
```

或使用 PHP 生成密码哈希：

```php
echo password_hash('新密码', PASSWORD_DEFAULT);
```

### Q: 如何添加新管理员？

```sql
INSERT INTO da_admins (username, password, nickname, status)
VALUES ('newadmin', '$2y$10$...', '新管理员', 1);
```

### Q: 日志文件在哪里？

日志文件位于 `server/logs/` 目录，按日期命名。
