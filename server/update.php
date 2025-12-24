<?php
/**
 * DevAssistant Tauri 自动更新接口
 *
 * 这是一个简单的 PHP 更新服务器，用于为 Tauri 应用提供自动更新功能。
 *
 * 使用方法：
 * 1. 将此文件和 version.json 上传到您的 Web 服务器
 * 2. 在 tauri.conf.json 中配置更新 URL：
 *    "updater": {
 *      "active": true,
 *      "endpoints": ["https://your-server.com/server/update.php"]
 *    }
 * 3. 每次发布新版本时，更新 version.json 文件
 *
 * 部署说明：
 * - 确保服务器支持 PHP 5.4+
 * - 确保 version.json 文件可读
 * - 建议使用 HTTPS 确保更新安全性
 * - 建议配置 CORS 允许跨域请求
 *
 * API 参数：
 * - target: 平台标识（如 windows-x86_64, darwin-x86_64 等）
 * - current_version: 当前安装的版本号（如 0.1.0）
 *
 * 返回值：
 * - 200: 有新版本可用，返回 JSON 格式的版本信息
 * - 204: 已是最新版本，无需更新
 * - 400: 请求参数错误
 * - 404: 未找到指定平台的版本信息
 * - 500: 服务器内部错误
 */

// 设置响应头
header('Content-Type: application/json; charset=utf-8');
header('Access-Control-Allow-Origin: *');
header('Access-Control-Allow-Methods: GET, OPTIONS');
header('Access-Control-Allow-Headers: Content-Type');

// 处理 OPTIONS 预检请求
if ($_SERVER['REQUEST_METHOD'] === 'OPTIONS') {
    http_response_code(200);
    exit;
}

// 只接受 GET 请求
if ($_SERVER['REQUEST_METHOD'] !== 'GET') {
    http_response_code(405);
    echo json_encode(['error' => 'Method Not Allowed']);
    exit;
}

/**
 * 比较版本号
 *
 * @param string $version1 版本号1（如 "0.1.0"）
 * @param string $version2 版本号2（如 "0.2.0"）
 * @return int 如果 version1 < version2 返回 -1，相等返回 0，version1 > version2 返回 1
 */
function compareVersions($version1, $version2) {
    // 移除可能的 'v' 前缀
    $version1 = ltrim($version1, 'vV');
    $version2 = ltrim($version2, 'vV');

    // 分割版本号
    $parts1 = explode('.', $version1);
    $parts2 = explode('.', $version2);

    // 补齐版本号位数
    $maxLength = max(count($parts1), count($parts2));
    $parts1 = array_pad($parts1, $maxLength, 0);
    $parts2 = array_pad($parts2, $maxLength, 0);

    // 逐位比较
    for ($i = 0; $i < $maxLength; $i++) {
        $num1 = intval($parts1[$i]);
        $num2 = intval($parts2[$i]);

        if ($num1 < $num2) {
            return -1;
        } elseif ($num1 > $num2) {
            return 1;
        }
    }

    return 0;
}

/**
 * 记录日志（可选功能）
 *
 * @param string $message 日志消息
 */
function logMessage($message) {
    $logFile = __DIR__ . '/update.log';
    $timestamp = date('Y-m-d H:i:s');
    $logEntry = "[{$timestamp}] {$message}\n";

    // 启用日志记录
    file_put_contents($logFile, $logEntry, FILE_APPEND);
}

try {
    // 获取请求参数
    $target = isset($_GET['target']) ? trim($_GET['target']) : '';
    $currentVersion = isset($_GET['current_version']) ? trim($_GET['current_version']) : '';

    // 记录详细请求信息
    $userAgent = isset($_SERVER['HTTP_USER_AGENT']) ? $_SERVER['HTTP_USER_AGENT'] : 'unknown';
    $requestUri = isset($_SERVER['REQUEST_URI']) ? $_SERVER['REQUEST_URI'] : 'unknown';
    logMessage("========== 新请求 ==========");
    logMessage("Request URI: {$requestUri}");
    logMessage("User-Agent: {$userAgent}");
    logMessage("target={$target}, current_version={$currentVersion}");

    // 验证参数
    if (empty($target)) {
        http_response_code(400);
        echo json_encode(['error' => 'Missing required parameter: target']);
        logMessage("Error: Missing target parameter");
        exit;
    }

    if (empty($currentVersion)) {
        http_response_code(400);
        echo json_encode(['error' => 'Missing required parameter: current_version']);
        logMessage("Error: Missing current_version parameter");
        exit;
    }

    // 读取版本配置文件
    $versionFile = __DIR__ . '/version.json';

    if (!file_exists($versionFile)) {
        http_response_code(500);
        echo json_encode(['error' => 'Version configuration file not found']);
        logMessage("Error: version.json not found");
        exit;
    }

    $versionData = json_decode(file_get_contents($versionFile), true);

    if ($versionData === null) {
        http_response_code(500);
        echo json_encode(['error' => 'Invalid version configuration file']);
        logMessage("Error: Invalid JSON in version.json");
        exit;
    }

    // 获取最新版本号
    $latestVersion = isset($versionData['version']) ? $versionData['version'] : '';

    if (empty($latestVersion)) {
        http_response_code(500);
        echo json_encode(['error' => 'Version number not found in configuration']);
        logMessage("Error: No version number in configuration");
        exit;
    }

    // 比较版本号
    $comparison = compareVersions($currentVersion, $latestVersion);

    // 如果当前版本已经是最新版本或更高
    if ($comparison >= 0) {
        http_response_code(204);
        logMessage("No update needed: current={$currentVersion}, latest={$latestVersion}");
        exit;
    }

    // Tauri 2.0 发送简化的 target (如 windows)，但期望响应中使用完整架构名 (如 windows-x86_64)
    // 定义 target 映射关系
    $targetMapping = [
        'windows' => 'windows-x86_64',
        'darwin' => 'darwin-x86_64',
        'linux' => 'linux-x86_64',
    ];

    // 确定实际的平台 key
    $platformKey = $target;
    if (isset($targetMapping[$target]) && isset($versionData['platforms'][$targetMapping[$target]])) {
        $platformKey = $targetMapping[$target];
        logMessage("Mapped target {$target} to {$platformKey}");
    }

    // 检查是否有对应平台的更新
    if (!isset($versionData['platforms'][$platformKey])) {
        http_response_code(404);
        echo json_encode(['error' => "No update available for platform: {$target}"]);
        logMessage("Error: Platform {$platformKey} not found in configuration");
        exit;
    }

    // 构建返回的更新信息 - 使用完整的架构名作为 key
    $updateInfo = [
        'version' => $versionData['version'],
        'notes' => isset($versionData['notes']) ? $versionData['notes'] : '',
        'pub_date' => isset($versionData['pub_date']) ? $versionData['pub_date'] : date('c'),
        'forceUpdate' => isset($versionData['forceUpdate']) ? (bool)$versionData['forceUpdate'] : false,
        'platforms' => [
            $platformKey => $versionData['platforms'][$platformKey]
        ]
    ];

    // 验证必要的平台信息
    if (empty($updateInfo['platforms'][$platformKey]['url'])) {
        http_response_code(500);
        echo json_encode(['error' => "Invalid update URL for platform: {$target}"]);
        logMessage("Error: Missing URL for platform {$target}");
        exit;
    }

    // 返回更新信息
    http_response_code(200);
    echo json_encode($updateInfo, JSON_UNESCAPED_UNICODE | JSON_UNESCAPED_SLASHES | JSON_PRETTY_PRINT);
    logMessage("Update available: {$currentVersion} -> {$latestVersion} for {$target}");

} catch (Exception $e) {
    // 捕获异常
    http_response_code(500);
    echo json_encode(['error' => 'Internal server error: ' . $e->getMessage()]);
    logMessage("Exception: " . $e->getMessage());
}
