<?php
/**
 * DevAssistant 公共函数库
 */

// 禁止直接访问
if (!defined('DA_ROOT')) {
    die('Access Denied');
}

/**
 * 获取数据库连接
 *
 * @return PDO
 */
function getDB() {
    static $pdo = null;

    if ($pdo === null) {
        $configFile = DA_ROOT . '/config/database.php';

        if (!file_exists($configFile)) {
            die('错误: 找不到数据库配置文件，路径: ' . $configFile);
        }

        $config = require $configFile;

        $dsn = sprintf(
            'mysql:host=%s;port=%d;dbname=%s;charset=%s',
            $config['host'],
            $config['port'],
            $config['database'],
            $config['charset']
        );

        try {
            $pdo = new PDO($dsn, $config['username'], $config['password'], [
                PDO::ATTR_ERRMODE => PDO::ERRMODE_EXCEPTION,
                PDO::ATTR_DEFAULT_FETCH_MODE => PDO::FETCH_ASSOC,
                PDO::ATTR_EMULATE_PREPARES => false,
            ]);
        } catch (PDOException $e) {
            // 显示详细错误信息（调试用）
            die('数据库连接失败: ' . $e->getMessage() . '<br>DSN: ' . $dsn);
        }
    }

    return $pdo;
}

/**
 * 返回 JSON 响应
 *
 * @param int $code 状态码
 * @param string $message 消息
 * @param array $data 数据
 */
function jsonResponse($code, $message = '', $data = []) {
    header('Content-Type: application/json; charset=utf-8');
    header('Access-Control-Allow-Origin: *');
    header('Access-Control-Allow-Methods: GET, POST, OPTIONS');
    header('Access-Control-Allow-Headers: Content-Type, Authorization');

    http_response_code($code);

    $response = [
        'code' => $code,
        'message' => $message,
    ];

    if (!empty($data)) {
        $response['data'] = $data;
    }

    echo json_encode($response, JSON_UNESCAPED_UNICODE);
    exit;
}

/**
 * 获取客户端 IP 地址
 *
 * @return string
 */
function getClientIP() {
    $ip = '';

    if (!empty($_SERVER['HTTP_CF_CONNECTING_IP'])) {
        // Cloudflare
        $ip = $_SERVER['HTTP_CF_CONNECTING_IP'];
    } elseif (!empty($_SERVER['HTTP_X_FORWARDED_FOR'])) {
        // 代理
        $ips = explode(',', $_SERVER['HTTP_X_FORWARDED_FOR']);
        $ip = trim($ips[0]);
    } elseif (!empty($_SERVER['HTTP_X_REAL_IP'])) {
        $ip = $_SERVER['HTTP_X_REAL_IP'];
    } elseif (!empty($_SERVER['REMOTE_ADDR'])) {
        $ip = $_SERVER['REMOTE_ADDR'];
    }

    return filter_var($ip, FILTER_VALIDATE_IP) ? $ip : 'unknown';
}

/**
 * 获取 POST JSON 数据
 *
 * @return array
 */
function getJsonInput() {
    $input = file_get_contents('php://input');
    $data = json_decode($input, true);
    return is_array($data) ? $data : [];
}

/**
 * 验证必填字段
 *
 * @param array $data 数据
 * @param array $required 必填字段
 * @return bool
 */
function validateRequired($data, $required) {
    foreach ($required as $field) {
        if (!isset($data[$field]) || $data[$field] === '') {
            return false;
        }
    }
    return true;
}

/**
 * 记录日志
 *
 * @param string $message 日志消息
 * @param string $type 日志类型
 */
function logMessage($message, $type = 'info') {
    $logDir = DA_ROOT . '/logs';
    if (!is_dir($logDir)) {
        mkdir($logDir, 0755, true);
    }

    $logFile = $logDir . '/' . date('Y-m-d') . '.log';
    $timestamp = date('Y-m-d H:i:s');
    $logEntry = "[{$timestamp}] [{$type}] {$message}\n";

    file_put_contents($logFile, $logEntry, FILE_APPEND);
}

/**
 * 生成设备唯一标识
 *
 * @param string $macAddress MAC地址
 * @return string
 */
function generateDeviceId($macAddress) {
    return md5(strtoupper(str_replace(['-', ':'], '', $macAddress)));
}

/**
 * 处理 OPTIONS 预检请求
 */
function handleCORS() {
    header('Access-Control-Allow-Origin: *');
    header('Access-Control-Allow-Methods: GET, POST, OPTIONS');
    header('Access-Control-Allow-Headers: Content-Type, Authorization');

    if ($_SERVER['REQUEST_METHOD'] === 'OPTIONS') {
        http_response_code(200);
        exit;
    }
}
