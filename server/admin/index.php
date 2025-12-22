<?php
/**
 * DevAssistant 管理后台 - 首页
 */

define('DA_ROOT', dirname(__DIR__));
require_once DA_ROOT . '/includes/common.php';
require_once DA_ROOT . '/includes/auth.php';

requireAdmin();

// 获取统计数据
$db = getDB();

// 总用户数
$stmt = $db->query('SELECT COUNT(*) as total FROM da_devices');
$totalDevices = $stmt->fetch()['total'];

// 活跃用户数(24小时内)
$stmt = $db->query('SELECT COUNT(*) as total FROM da_devices WHERE last_seen >= DATE_SUB(NOW(), INTERVAL 24 HOUR)');
$activeDevices = $stmt->fetch()['total'];

// 禁用用户数
$stmt = $db->query('SELECT COUNT(*) as total FROM da_devices WHERE status = 0');
$bannedDevices = $stmt->fetch()['total'];

// 待处理申诉
$stmt = $db->query('SELECT COUNT(*) as total FROM da_appeals WHERE status = 0');
$pendingAppeals = $stmt->fetch()['total'];

// 今日上报次数
$stmt = $db->query('SELECT COUNT(*) as total FROM da_heartbeats WHERE DATE(report_time) = CURDATE()');
$todayHeartbeats = $stmt->fetch()['total'];
?>
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>DevAssistant 管理后台</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #f5f7fa;
            min-height: 100vh;
        }
        .header {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 15px 30px;
            display: flex;
            justify-content: space-between;
            align-items: center;
        }
        .header h1 { font-size: 20px; }
        .header a { color: white; text-decoration: none; }
        .nav {
            background: white;
            padding: 0 30px;
            border-bottom: 1px solid #e0e0e0;
        }
        .nav a {
            display: inline-block;
            padding: 15px 20px;
            color: #666;
            text-decoration: none;
            border-bottom: 2px solid transparent;
        }
        .nav a:hover, .nav a.active {
            color: #667eea;
            border-bottom-color: #667eea;
        }
        .container {
            max-width: 1200px;
            margin: 0 auto;
            padding: 30px;
        }
        .stats {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 20px;
            margin-bottom: 30px;
        }
        .stat-card {
            background: white;
            padding: 25px;
            border-radius: 10px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.05);
        }
        .stat-card h3 {
            color: #888;
            font-size: 14px;
            margin-bottom: 10px;
        }
        .stat-card .value {
            font-size: 32px;
            font-weight: bold;
            color: #333;
        }
        .stat-card.warning .value { color: #f59e0b; }
        .stat-card.danger .value { color: #ef4444; }
        .stat-card.success .value { color: #10b981; }
        .card {
            background: white;
            border-radius: 10px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.05);
            margin-bottom: 20px;
        }
        .card-header {
            padding: 15px 20px;
            border-bottom: 1px solid #eee;
            font-weight: bold;
        }
        .card-body { padding: 20px; }
        table {
            width: 100%;
            border-collapse: collapse;
        }
        table th, table td {
            padding: 12px 15px;
            text-align: left;
            border-bottom: 1px solid #eee;
        }
        table th {
            background: #f8f9fa;
            font-weight: 500;
            color: #666;
        }
        .badge {
            display: inline-block;
            padding: 4px 10px;
            border-radius: 20px;
            font-size: 12px;
        }
        .badge-success { background: #d1fae5; color: #059669; }
        .badge-danger { background: #fee2e2; color: #dc2626; }
        .badge-warning { background: #fef3c7; color: #d97706; }
    </style>
</head>
<body>
    <div class="header">
        <h1>DevAssistant 管理后台</h1>
        <div>
            <span>欢迎, <?php echo htmlspecialchars($_SESSION['admin_nickname'] ?: $_SESSION['admin_username']); ?></span>
            &nbsp;|&nbsp;
            <a href="logout.php">退出</a>
        </div>
    </div>

    <div class="nav">
        <a href="index.php" class="active">概览</a>
        <a href="devices.php">设备管理</a>
        <a href="appeals.php">申诉管理</a>
        <a href="logs.php">操作日志</a>
    </div>

    <div class="container">
        <div class="stats">
            <div class="stat-card">
                <h3>总设备数</h3>
                <div class="value"><?php echo number_format($totalDevices); ?></div>
            </div>
            <div class="stat-card success">
                <h3>活跃设备 (24h)</h3>
                <div class="value"><?php echo number_format($activeDevices); ?></div>
            </div>
            <div class="stat-card danger">
                <h3>已禁用</h3>
                <div class="value"><?php echo number_format($bannedDevices); ?></div>
            </div>
            <div class="stat-card warning">
                <h3>待处理申诉</h3>
                <div class="value"><?php echo number_format($pendingAppeals); ?></div>
            </div>
            <div class="stat-card">
                <h3>今日上报</h3>
                <div class="value"><?php echo number_format($todayHeartbeats); ?></div>
            </div>
        </div>

        <?php
        // 最近活跃设备
        $stmt = $db->query('SELECT * FROM da_devices ORDER BY last_seen DESC LIMIT 10');
        $recentDevices = $stmt->fetchAll();
        ?>

        <div class="card">
            <div class="card-header">最近活跃设备</div>
            <div class="card-body">
                <table>
                    <thead>
                        <tr>
                            <th>计算机名</th>
                            <th>用户名</th>
                            <th>MAC地址</th>
                            <th>版本</th>
                            <th>最后活跃</th>
                            <th>状态</th>
                        </tr>
                    </thead>
                    <tbody>
                        <?php foreach ($recentDevices as $device): ?>
                        <tr>
                            <td><?php echo htmlspecialchars($device['hostname'] ?: '-'); ?></td>
                            <td><?php echo htmlspecialchars($device['username'] ?: '-'); ?></td>
                            <td><?php echo htmlspecialchars($device['mac_address']); ?></td>
                            <td><?php echo htmlspecialchars($device['app_version'] ?: '-'); ?></td>
                            <td><?php echo $device['last_seen']; ?></td>
                            <td>
                                <?php if ($device['status'] == 1): ?>
                                    <span class="badge badge-success">正常</span>
                                <?php else: ?>
                                    <span class="badge badge-danger">禁用</span>
                                <?php endif; ?>
                            </td>
                        </tr>
                        <?php endforeach; ?>
                    </tbody>
                </table>
            </div>
        </div>
    </div>
</body>
</html>
