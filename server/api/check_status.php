<?php
/**
 * DevAssistant 状态检查接口
 *
 * 检查设备是否被禁用
 *
 * 请求方式: POST
 * 请求参数 (JSON):
 *   - mac_address: string (必填) MAC地址
 *
 * 响应:
 *   - 200: 状态正常
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
$deviceId = generateDeviceId($macAddress);

try {
    $db = getDB();

    // 查询设备状态
    $stmt = $db->prepare('
        SELECT status, ban_reason, ban_time
        FROM da_devices
        WHERE device_id = ?
    ');
    $stmt->execute([$deviceId]);
    $device = $stmt->fetch();

    if (!$device) {
        // 设备不存在，视为正常（新用户）
        jsonResponse(200, '状态正常', [
            'status' => 'active',
            'is_new' => true,
        ]);
    }

    if ($device['status'] == 0) {
        // 检查是否有待处理的申诉
        $stmt = $db->prepare('
            SELECT id, status as appeal_status
            FROM da_appeals
            WHERE device_id = ? AND status = 0
            ORDER BY created_at DESC
            LIMIT 1
        ');
        $stmt->execute([$deviceId]);
        $appeal = $stmt->fetch();

        jsonResponse(403, '您的设备已被禁用', [
            'status' => 'banned',
            'reason' => $device['ban_reason'] ?: '违反使用条款',
            'ban_time' => $device['ban_time'],
            'can_appeal' => true,
            'has_pending_appeal' => !empty($appeal),
        ]);
    }

    jsonResponse(200, '状态正常', [
        'status' => 'active',
    ]);

} catch (PDOException $e) {
    logMessage('Database error: ' . $e->getMessage(), 'error');
    jsonResponse(500, '服务器内部错误');
}
