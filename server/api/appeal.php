<?php
/**
 * DevAssistant 申诉接口
 *
 * 用户提交解封申诉
 *
 * 请求方式: POST
 * 请求参数 (JSON):
 *   - mac_address: string (必填) MAC地址
 *   - contact: string (必填) 联系方式(邮箱或电话)
 *   - reason: string (必填) 申诉理由
 *
 * 响应:
 *   - 200: 申诉提交成功
 *   - 400: 参数错误
 *   - 409: 已有待处理的申诉
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
if (!validateRequired($data, ['mac_address', 'contact', 'reason'])) {
    jsonResponse(400, '缺少必填参数');
}

$macAddress = trim($data['mac_address']);
$contact = trim($data['contact']);
$reason = trim($data['reason']);

// 验证联系方式格式
if (strlen($contact) < 5) {
    jsonResponse(400, '请填写有效的联系方式');
}

// 验证申诉理由长度
if (strlen($reason) < 10) {
    jsonResponse(400, '申诉理由至少需要10个字符');
}

if (strlen($reason) > 2000) {
    jsonResponse(400, '申诉理由不能超过2000个字符');
}

$deviceId = generateDeviceId($macAddress);

try {
    $db = getDB();

    // 检查设备是否存在且被禁用
    $stmt = $db->prepare('SELECT status FROM da_devices WHERE device_id = ?');
    $stmt->execute([$deviceId]);
    $device = $stmt->fetch();

    if (!$device) {
        jsonResponse(400, '设备信息不存在');
    }

    if ($device['status'] == 1) {
        jsonResponse(400, '您的设备状态正常，无需申诉');
    }

    // 检查是否已有待处理的申诉
    $stmt = $db->prepare('
        SELECT id, created_at
        FROM da_appeals
        WHERE device_id = ? AND status = 0
        ORDER BY created_at DESC
        LIMIT 1
    ');
    $stmt->execute([$deviceId]);
    $existingAppeal = $stmt->fetch();

    if ($existingAppeal) {
        jsonResponse(409, '您已有待处理的申诉，请耐心等待', [
            'appeal_id' => $existingAppeal['id'],
            'submitted_at' => $existingAppeal['created_at'],
        ]);
    }

    // 提交申诉
    $stmt = $db->prepare('
        INSERT INTO da_appeals (device_id, contact, reason, status)
        VALUES (?, ?, ?, 0)
    ');
    $stmt->execute([$deviceId, $contact, $reason]);

    $appealId = $db->lastInsertId();

    // 记录日志
    logMessage("Appeal submitted: device={$deviceId}, appeal_id={$appealId}");

    jsonResponse(200, '申诉提交成功，我们会尽快处理', [
        'appeal_id' => $appealId,
    ]);

} catch (PDOException $e) {
    logMessage('Database error: ' . $e->getMessage(), 'error');
    jsonResponse(500, '服务器内部错误');
}
