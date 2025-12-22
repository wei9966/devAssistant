<?php
/**
 * DevAssistant 心跳上报接口
 *
 * 接收客户端上报的设备信息，记录用户使用情况
 *
 * 请求方式: POST
 * 请求参数 (JSON):
 *   - mac_address: string (必填) MAC地址
 *   - hostname: string (可选) 计算机名
 *   - username: string (可选) 系统用户名
 *   - os_info: string (可选) 操作系统信息
 *   - app_version: string (可选) 软件版本
 *   - local_ip: string (可选) 本地IP地址
 *
 * 响应:
 *   - 200: 上报成功
 *   - 400: 参数错误
 *   - 403: 设备已被禁用
 *   - 500: 服务器错误
 */

define('DA_ROOT', dirname(__DIR__));
require_once DA_ROOT . '/includes/common.php';

// 处理 CORS
handleCORS();

// 只接受 POST 请求
if ($_SERVER['REQUEST_METHOD'] !== 'POST') {
    jsonResponse(405, '请求方法不允许');
}

// 获取请求数据
$data = getJsonInput();

// 验证必填字段
if (!validateRequired($data, ['mac_address'])) {
    jsonResponse(400, '缺少必填参数: mac_address');
}

$macAddress = trim($data['mac_address']);
$hostname = isset($data['hostname']) ? trim($data['hostname']) : '';
$username = isset($data['username']) ? trim($data['username']) : '';
$osInfo = isset($data['os_info']) ? trim($data['os_info']) : '';
$appVersion = isset($data['app_version']) ? trim($data['app_version']) : '';
$localIp = isset($data['local_ip']) ? trim($data['local_ip']) : '';
$extraData = isset($data['extra']) ? $data['extra'] : null;

// 生成设备唯一标识
$deviceId = generateDeviceId($macAddress);

// 获取客户端 IP
$clientIp = getClientIP();

try {
    $db = getDB();
    $now = date('Y-m-d H:i:s');

    // 检查设备是否存在
    $stmt = $db->prepare('SELECT id, status, ban_reason FROM da_devices WHERE device_id = ?');
    $stmt->execute([$deviceId]);
    $device = $stmt->fetch();

    if ($device) {
        // 检查是否被禁用
        if ($device['status'] == 0) {
            jsonResponse(403, '您的设备已被禁用', [
                'banned' => true,
                'reason' => $device['ban_reason'] ?: '违反使用条款',
                'can_appeal' => true,
            ]);
        }

        // 更新设备信息
        $stmt = $db->prepare('
            UPDATE da_devices SET
                hostname = COALESCE(NULLIF(?, ""), hostname),
                username = COALESCE(NULLIF(?, ""), username),
                os_info = COALESCE(NULLIF(?, ""), os_info),
                app_version = COALESCE(NULLIF(?, ""), app_version),
                last_seen = ?,
                total_reports = total_reports + 1
            WHERE device_id = ?
        ');
        $stmt->execute([$hostname, $username, $osInfo, $appVersion, $now, $deviceId]);
    } else {
        // 新设备，插入记录
        $stmt = $db->prepare('
            INSERT INTO da_devices
                (device_id, mac_address, hostname, username, os_info, app_version, first_seen, last_seen, status)
            VALUES
                (?, ?, ?, ?, ?, ?, ?, ?, 1)
        ');
        $stmt->execute([$deviceId, $macAddress, $hostname, $username, $osInfo, $appVersion, $now, $now]);
    }

    // 记录心跳
    $stmt = $db->prepare('
        INSERT INTO da_heartbeats
            (device_id, ip_address, local_ip, app_version, report_time, extra_data)
        VALUES
            (?, ?, ?, ?, ?, ?)
    ');
    $stmt->execute([
        $deviceId,
        $clientIp,
        $localIp,
        $appVersion,
        $now,
        $extraData ? json_encode($extraData) : null,
    ]);

    // 记录日志
    logMessage("Heartbeat: device={$deviceId}, ip={$clientIp}, version={$appVersion}");

    jsonResponse(200, '上报成功', [
        'device_id' => $deviceId,
        'status' => 'active',
    ]);

} catch (PDOException $e) {
    logMessage('Database error: ' . $e->getMessage(), 'error');
    jsonResponse(500, '服务器内部错误');
}
