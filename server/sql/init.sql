-- =============================================
-- DevAssistant 用户追踪系统 - 数据库初始化脚本
-- =============================================
-- 执行前请先创建数据库: CREATE DATABASE dev_assistant CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
-- =============================================

-- 用户设备表 - 存储所有上报的设备信息
CREATE TABLE IF NOT EXISTS `da_devices` (
    `id` INT UNSIGNED AUTO_INCREMENT PRIMARY KEY COMMENT '主键ID',
    `device_id` VARCHAR(64) NOT NULL COMMENT '设备唯一标识(MAC地址MD5)',
    `mac_address` VARCHAR(64) NOT NULL COMMENT 'MAC地址',
    `hostname` VARCHAR(128) DEFAULT '' COMMENT '计算机名',
    `username` VARCHAR(128) DEFAULT '' COMMENT '系统用户名',
    `os_info` VARCHAR(256) DEFAULT '' COMMENT '操作系统信息',
    `app_version` VARCHAR(32) DEFAULT '' COMMENT '软件版本',
    `first_seen` DATETIME NOT NULL COMMENT '首次上报时间',
    `last_seen` DATETIME NOT NULL COMMENT '最后活跃时间',
    `total_reports` INT UNSIGNED DEFAULT 1 COMMENT '累计上报次数',
    `status` TINYINT DEFAULT 1 COMMENT '状态: 1=正常, 0=禁用',
    `ban_reason` VARCHAR(512) DEFAULT '' COMMENT '禁用原因',
    `ban_time` DATETIME DEFAULT NULL COMMENT '禁用时间',
    `remark` VARCHAR(512) DEFAULT '' COMMENT '备注',
    `created_at` TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    `updated_at` TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    UNIQUE KEY `uk_device_id` (`device_id`),
    KEY `idx_mac_address` (`mac_address`),
    KEY `idx_status` (`status`),
    KEY `idx_last_seen` (`last_seen`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='用户设备表';

-- 心跳记录表 - 存储每次上报的详细信息
CREATE TABLE IF NOT EXISTS `da_heartbeats` (
    `id` BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY COMMENT '主键ID',
    `device_id` VARCHAR(64) NOT NULL COMMENT '设备唯一标识',
    `ip_address` VARCHAR(64) NOT NULL COMMENT 'IP地址',
    `local_ip` VARCHAR(64) DEFAULT '' COMMENT '本地IP地址',
    `app_version` VARCHAR(32) DEFAULT '' COMMENT '软件版本',
    `report_time` DATETIME NOT NULL COMMENT '上报时间',
    `extra_data` JSON DEFAULT NULL COMMENT '额外数据(JSON格式)',
    `created_at` TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    KEY `idx_device_id` (`device_id`),
    KEY `idx_report_time` (`report_time`),
    KEY `idx_ip_address` (`ip_address`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='心跳记录表';

-- 申诉记录表 - 存储用户的申诉信息
CREATE TABLE IF NOT EXISTS `da_appeals` (
    `id` INT UNSIGNED AUTO_INCREMENT PRIMARY KEY COMMENT '主键ID',
    `device_id` VARCHAR(64) NOT NULL COMMENT '设备唯一标识',
    `contact` VARCHAR(256) NOT NULL COMMENT '联系方式(邮箱/电话)',
    `reason` TEXT NOT NULL COMMENT '申诉理由',
    `status` TINYINT DEFAULT 0 COMMENT '状态: 0=待处理, 1=已通过, 2=已拒绝',
    `admin_reply` TEXT DEFAULT NULL COMMENT '管理员回复',
    `process_time` DATETIME DEFAULT NULL COMMENT '处理时间',
    `created_at` TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    `updated_at` TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    KEY `idx_device_id` (`device_id`),
    KEY `idx_status` (`status`),
    KEY `idx_created_at` (`created_at`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='申诉记录表';

-- 管理员表 - 存储后台管理员账号
CREATE TABLE IF NOT EXISTS `da_admins` (
    `id` INT UNSIGNED AUTO_INCREMENT PRIMARY KEY COMMENT '主键ID',
    `username` VARCHAR(64) NOT NULL COMMENT '用户名',
    `password` VARCHAR(255) NOT NULL COMMENT '密码(加密存储)',
    `nickname` VARCHAR(64) DEFAULT '' COMMENT '昵称',
    `status` TINYINT DEFAULT 1 COMMENT '状态: 1=正常, 0=禁用',
    `last_login` DATETIME DEFAULT NULL COMMENT '最后登录时间',
    `last_login_ip` VARCHAR(64) DEFAULT '' COMMENT '最后登录IP',
    `created_at` TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    `updated_at` TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    UNIQUE KEY `uk_username` (`username`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='管理员表';

-- 操作日志表 - 记录管理员操作
CREATE TABLE IF NOT EXISTS `da_operation_logs` (
    `id` BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY COMMENT '主键ID',
    `admin_id` INT UNSIGNED NOT NULL COMMENT '管理员ID',
    `action` VARCHAR(64) NOT NULL COMMENT '操作类型',
    `target_type` VARCHAR(32) DEFAULT '' COMMENT '目标类型',
    `target_id` VARCHAR(64) DEFAULT '' COMMENT '目标ID',
    `detail` TEXT DEFAULT NULL COMMENT '操作详情',
    `ip_address` VARCHAR(64) DEFAULT '' COMMENT '操作IP',
    `created_at` TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    KEY `idx_admin_id` (`admin_id`),
    KEY `idx_action` (`action`),
    KEY `idx_created_at` (`created_at`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='操作日志表';

-- 插入默认管理员账号 (用户名: admin, 密码: admin123)
-- 密码使用 password_hash() 加密，请在正式使用前修改密码
INSERT INTO `da_admins` (`username`, `password`, `nickname`, `status`) VALUES
('admin', '$2y$10$N9qo8uLOickgx2ZMRZoMye1YzAN0VuKSJX5X2KnWJCXBn.bCwWKbO', '管理员', 1)
ON DUPLICATE KEY UPDATE `password` = '$2y$10$N9qo8uLOickgx2ZMRZoMye1YzAN0VuKSJX5X2KnWJCXBn.bCwWKbO', `updated_at` = CURRENT_TIMESTAMP;
