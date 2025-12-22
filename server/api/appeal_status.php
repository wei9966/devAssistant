<?php
/**
 * DevAssistant 申诉状态查询接口
 *
 * 查询申诉处理状态
 *
 * 请求方式: POST
 * 请求参数 (JSON):
 *   - mac_address: string (必填) MAC地址
 *
 * 响应:
 *   - 200: 查询成功
 *   - 400: 参数错误
 *   - 404: 无申诉记录
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

    // 查询最近的申诉记录
    $stmt = $db->prepare('
        SELECT id, status, admin_reply, process_time, created_at
        FROM da_appeals
        WHERE device_id = ?
        ORDER BY created_at DESC
        LIMIT 5
    ');
    $stmt->execute([$deviceId]);
    $appeals = $stmt->fetchAll();

    if (empty($appeals)) {
        jsonResponse(404, '暂无申诉记录');
    }

    $statusMap = [
        0 => '待处理',
        1 => '已通过',
        2 => '已拒绝',
    ];

    $result = array_map(function ($appeal) use ($statusMap) {
        return [
            'id' => $appeal['id'],
            'status' => $appeal['status'],
            'status_text' => $statusMap[$appeal['status']] ?? '未知',
            'admin_reply' => $appeal['admin_reply'],
            'process_time' => $appeal['process_time'],
            'created_at' => $appeal['created_at'],
        ];
    }, $appeals);

    jsonResponse(200, '查询成功', [
        'appeals' => $result,
    ]);

} catch (PDOException $e) {
    logMessage('Database error: ' . $e->getMessage(), 'error');
    jsonResponse(500, '服务器内部错误');
}
