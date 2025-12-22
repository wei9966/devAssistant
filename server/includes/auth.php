<?php
/**
 * DevAssistant 管理员认证模块
 */

// 禁止直接访问
if (!defined('DA_ROOT')) {
    die('Access Denied');
}

// 开启 session
if (session_status() === PHP_SESSION_NONE) {
    session_start();
}

/**
 * 检查管理员是否已登录
 *
 * @return bool
 */
function isAdminLoggedIn() {
    return !empty($_SESSION['admin_id']) && !empty($_SESSION['admin_username']);
}

/**
 * 获取当前登录的管理员ID
 *
 * @return int|null
 */
function getAdminId() {
    return $_SESSION['admin_id'] ?? null;
}

/**
 * 获取当前登录的管理员用户名
 *
 * @return string|null
 */
function getAdminUsername() {
    return $_SESSION['admin_username'] ?? null;
}

/**
 * 要求管理员登录
 *
 * @param bool $isApi 是否为 API 请求
 */
function requireAdmin($isApi = false) {
    if (!isAdminLoggedIn()) {
        if ($isApi) {
            jsonResponse(401, '请先登录');
        } else {
            header('Location: login.php');
            exit;
        }
    }
}

/**
 * 管理员登录
 *
 * @param string $username 用户名
 * @param string $password 密码
 * @return array
 */
function adminLogin($username, $password) {
    $db = getDB();

    $stmt = $db->prepare('SELECT id, username, password, nickname, status FROM da_admins WHERE username = ?');
    $stmt->execute([$username]);
    $admin = $stmt->fetch();

    if (!$admin) {
        return ['success' => false, 'message' => '用户名或密码错误'];
    }

    if ($admin['status'] != 1) {
        return ['success' => false, 'message' => '账号已被禁用'];
    }

    if (!password_verify($password, $admin['password'])) {
        return ['success' => false, 'message' => '用户名或密码错误'];
    }

    // 更新登录信息
    $stmt = $db->prepare('UPDATE da_admins SET last_login = NOW(), last_login_ip = ? WHERE id = ?');
    $stmt->execute([getClientIP(), $admin['id']]);

    // 设置 session
    $_SESSION['admin_id'] = $admin['id'];
    $_SESSION['admin_username'] = $admin['username'];
    $_SESSION['admin_nickname'] = $admin['nickname'];

    return ['success' => true, 'message' => '登录成功'];
}

/**
 * 管理员退出登录
 */
function adminLogout() {
    $_SESSION = [];
    session_destroy();
}

/**
 * 记录操作日志
 *
 * @param string $action 操作类型
 * @param string $targetType 目标类型
 * @param string $targetId 目标ID
 * @param string $detail 详情
 */
function logOperation($action, $targetType = '', $targetId = '', $detail = '') {
    $db = getDB();

    $stmt = $db->prepare('
        INSERT INTO da_operation_logs (admin_id, action, target_type, target_id, detail, ip_address)
        VALUES (?, ?, ?, ?, ?, ?)
    ');
    $stmt->execute([
        getAdminId(),
        $action,
        $targetType,
        $targetId,
        $detail,
        getClientIP(),
    ]);
}
