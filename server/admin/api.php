<?php
/**
 * DevAssistant 管理后台 API
 */

define('DA_ROOT', dirname(__DIR__));
require_once DA_ROOT . '/includes/common.php';
require_once DA_ROOT . '/includes/auth.php';

// 处理 CORS
handleCORS();

// 验证登录
requireAdmin(true);

$action = $_GET['action'] ?? '';
$data = getJsonInput();

try {
    $db = getDB();

    switch ($action) {
        case 'ban':
            // 禁用设备
            $deviceId = $data['device_id'] ?? '';
            $reason = $data['reason'] ?? '';

            if (empty($deviceId)) {
                jsonResponse(400, '缺少设备ID');
            }

            $stmt = $db->prepare('
                UPDATE da_devices
                SET status = 0, ban_reason = ?, ban_time = NOW()
                WHERE device_id = ?
            ');
            $stmt->execute([$reason, $deviceId]);

            logOperation('ban_device', 'device', $deviceId, $reason);
            logMessage("Device banned: {$deviceId}, reason: {$reason}");

            jsonResponse(200, '禁用成功');
            break;

        case 'unban':
            // 解禁设备
            $deviceId = $data['device_id'] ?? '';

            if (empty($deviceId)) {
                jsonResponse(400, '缺少设备ID');
            }

            $stmt = $db->prepare('
                UPDATE da_devices
                SET status = 1, ban_reason = "", ban_time = NULL
                WHERE device_id = ?
            ');
            $stmt->execute([$deviceId]);

            logOperation('unban_device', 'device', $deviceId, '管理员解禁');
            logMessage("Device unbanned: {$deviceId}");

            jsonResponse(200, '解禁成功');
            break;

        case 'approve_appeal':
            // 通过申诉
            $appealId = $data['appeal_id'] ?? 0;
            $reply = $data['reply'] ?? '申诉通过';

            if (empty($appealId)) {
                jsonResponse(400, '缺少申诉ID');
            }

            // 获取申诉信息
            $stmt = $db->prepare('SELECT device_id FROM da_appeals WHERE id = ? AND status = 0');
            $stmt->execute([$appealId]);
            $appeal = $stmt->fetch();

            if (!$appeal) {
                jsonResponse(400, '申诉不存在或已处理');
            }

            // 更新申诉状态
            $stmt = $db->prepare('
                UPDATE da_appeals
                SET status = 1, admin_reply = ?, process_time = NOW()
                WHERE id = ?
            ');
            $stmt->execute([$reply, $appealId]);

            // 解禁设备
            $stmt = $db->prepare('
                UPDATE da_devices
                SET status = 1, ban_reason = "", ban_time = NULL
                WHERE device_id = ?
            ');
            $stmt->execute([$appeal['device_id']]);

            logOperation('approve_appeal', 'appeal', $appealId, $reply);
            logMessage("Appeal approved: {$appealId}, device unbanned: {$appeal['device_id']}");

            jsonResponse(200, '申诉已通过，设备已解禁');
            break;

        case 'reject_appeal':
            // 拒绝申诉
            $appealId = $data['appeal_id'] ?? 0;
            $reply = $data['reply'] ?? '';

            if (empty($appealId)) {
                jsonResponse(400, '缺少申诉ID');
            }

            if (empty($reply)) {
                jsonResponse(400, '请填写拒绝理由');
            }

            $stmt = $db->prepare('
                UPDATE da_appeals
                SET status = 2, admin_reply = ?, process_time = NOW()
                WHERE id = ? AND status = 0
            ');
            $stmt->execute([$reply, $appealId]);

            if ($stmt->rowCount() === 0) {
                jsonResponse(400, '申诉不存在或已处理');
            }

            logOperation('reject_appeal', 'appeal', $appealId, $reply);
            logMessage("Appeal rejected: {$appealId}");

            jsonResponse(200, '申诉已拒绝');
            break;

        case 'device_detail':
            // 获取设备详情
            $deviceId = $_GET['device_id'] ?? '';

            if (empty($deviceId)) {
                jsonResponse(400, '缺少设备ID');
            }

            $stmt = $db->prepare('SELECT * FROM da_devices WHERE device_id = ?');
            $stmt->execute([$deviceId]);
            $device = $stmt->fetch();

            if (!$device) {
                jsonResponse(404, '设备不存在');
            }

            // 获取最近心跳记录
            $stmt = $db->prepare('
                SELECT * FROM da_heartbeats
                WHERE device_id = ?
                ORDER BY report_time DESC
                LIMIT 20
            ');
            $stmt->execute([$deviceId]);
            $heartbeats = $stmt->fetchAll();

            jsonResponse(200, '获取成功', [
                'device' => $device,
                'heartbeats' => $heartbeats,
            ]);
            break;

        case 'update_remark':
            // 更新设备备注
            $deviceId = $data['device_id'] ?? '';
            $remark = $data['remark'] ?? '';

            if (empty($deviceId)) {
                jsonResponse(400, '缺少设备ID');
            }

            $stmt = $db->prepare('UPDATE da_devices SET remark = ? WHERE device_id = ?');
            $stmt->execute([$remark, $deviceId]);

            logOperation('update_remark', 'device', $deviceId, $remark);

            jsonResponse(200, '备注更新成功');
            break;

        default:
            jsonResponse(400, '未知操作');
    }

} catch (PDOException $e) {
    logMessage('API Error: ' . $e->getMessage(), 'error');
    jsonResponse(500, '服务器内部错误');
}
