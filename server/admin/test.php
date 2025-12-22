<?php
/**
 * DevAssistant 服务器诊断测试
 *
 * 用于检查服务器环境是否正确配置
 * 测试完成后请删除此文件！
 */

ini_set('display_errors', 1);
ini_set('display_startup_errors', 1);
error_reporting(E_ALL);

echo "<h1>DevAssistant 服务器诊断</h1>";

// 1. PHP版本
echo "<h2>1. PHP 版本</h2>";
echo "<p>PHP Version: " . PHP_VERSION . "</p>";

// 2. 必需扩展
echo "<h2>2. 必需扩展</h2>";
$extensions = ['pdo', 'pdo_mysql', 'json', 'session'];
foreach ($extensions as $ext) {
    $status = extension_loaded($ext) ? '✅ 已安装' : '❌ 未安装';
    echo "<p>{$ext}: {$status}</p>";
}

// 3. 目录结构
echo "<h2>3. 目录结构</h2>";
$rootDir = dirname(__DIR__);
echo "<p>根目录: {$rootDir}</p>";

$checkDirs = [
    '/config' => '配置目录',
    '/includes' => '公共文件目录',
    '/api' => 'API目录',
    '/admin' => '管理后台目录',
];

foreach ($checkDirs as $dir => $name) {
    $path = $rootDir . $dir;
    $status = is_dir($path) ? '✅ 存在' : '❌ 不存在';
    echo "<p>{$name} ({$dir}): {$status}</p>";
}

// 4. 关键文件
echo "<h2>4. 关键文件</h2>";
$checkFiles = [
    '/config/database.php' => '数据库配置',
    '/includes/common.php' => '公共函数库',
    '/includes/auth.php' => '认证模块',
];

foreach ($checkFiles as $file => $name) {
    $path = $rootDir . $file;
    $status = file_exists($path) ? '✅ 存在' : '❌ 不存在';
    echo "<p>{$name} ({$file}): {$status}</p>";
}

// 5. 数据库连接测试
echo "<h2>5. 数据库连接测试</h2>";
$configFile = $rootDir . '/config/database.php';
if (file_exists($configFile)) {
    $config = require $configFile;
    echo "<p>Host: {$config['host']}</p>";
    echo "<p>Port: {$config['port']}</p>";
    echo "<p>Database: {$config['database']}</p>";
    echo "<p>Username: {$config['username']}</p>";
    echo "<p>Password: " . (empty($config['password']) ? '❌ 未设置' : '✅ 已设置') . "</p>";

    try {
        $dsn = sprintf(
            'mysql:host=%s;port=%d;dbname=%s;charset=%s',
            $config['host'],
            $config['port'],
            $config['database'],
            $config['charset']
        );

        echo "<p>DSN: {$dsn}</p>";

        $pdo = new PDO($dsn, $config['username'], $config['password'], [
            PDO::ATTR_ERRMODE => PDO::ERRMODE_EXCEPTION,
            PDO::ATTR_DEFAULT_FETCH_MODE => PDO::FETCH_ASSOC,
        ]);

        echo "<p style='color:green;font-weight:bold;'>✅ 数据库连接成功！</p>";

        // 检查表是否存在
        echo "<h3>数据库表检查</h3>";
        $tables = ['da_devices', 'da_heartbeats', 'da_appeals', 'da_admins', 'da_operation_logs'];
        foreach ($tables as $table) {
            try {
                $stmt = $pdo->query("SELECT 1 FROM {$table} LIMIT 1");
                echo "<p>{$table}: ✅ 存在</p>";
            } catch (PDOException $e) {
                echo "<p>{$table}: ❌ 不存在或无法访问</p>";
            }
        }

    } catch (PDOException $e) {
        echo "<p style='color:red;font-weight:bold;'>❌ 数据库连接失败: " . htmlspecialchars($e->getMessage()) . "</p>";
    }
} else {
    echo "<p style='color:red;'>❌ 数据库配置文件不存在</p>";
}

// 6. Session 测试
echo "<h2>6. Session 测试</h2>";
if (session_status() === PHP_SESSION_NONE) {
    session_start();
}
$_SESSION['test'] = 'ok';
echo "<p>Session 状态: " . (session_status() === PHP_SESSION_ACTIVE ? '✅ 正常' : '❌ 异常') . "</p>";

echo "<hr>";
echo "<p style='color:red;'><strong>安全提示：测试完成后请立即删除此文件！</strong></p>";
