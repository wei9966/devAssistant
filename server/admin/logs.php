<?php
/**
 * DevAssistant 管理后台 - 操作日志
 */

define('DA_ROOT', dirname(__DIR__));
require_once DA_ROOT . '/includes/common.php';
require_once DA_ROOT . '/includes/auth.php';

requireAdmin();

$db = getDB();

// 分页参数
$page = max(1, intval($_GET['page'] ?? 1));
$perPage = 30;
$offset = ($page - 1) * $perPage;

// 获取总数
$stmt = $db->query('SELECT COUNT(*) as total FROM da_operation_logs');
$total = $stmt->fetch()['total'];
$totalPages = ceil($total / $perPage);

// 获取列表
$stmt = $db->prepare("
    SELECT l.*, a.username, a.nickname
    FROM da_operation_logs l
    LEFT JOIN da_admins a ON l.admin_id = a.id
    ORDER BY l.created_at DESC
    LIMIT {$offset}, {$perPage}
");
$stmt->execute();
$logs = $stmt->fetchAll();

$actionMap = [
    'ban_device' => '禁用设备',
    'unban_device' => '解禁设备',
    'approve_appeal' => '通过申诉',
    'reject_appeal' => '拒绝申诉',
    'update_remark' => '更新备注',
];
?>
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>操作日志 - DevAssistant 管理后台</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #f5f7fa; min-height: 100vh; }
        .header { background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 15px 30px; display: flex; justify-content: space-between; align-items: center; }
        .header h1 { font-size: 20px; }
        .header a { color: white; text-decoration: none; }
        .nav { background: white; padding: 0 30px; border-bottom: 1px solid #e0e0e0; }
        .nav a { display: inline-block; padding: 15px 20px; color: #666; text-decoration: none; border-bottom: 2px solid transparent; }
        .nav a:hover, .nav a.active { color: #667eea; border-bottom-color: #667eea; }
        .container { max-width: 1200px; margin: 0 auto; padding: 30px; }
        .card { background: white; border-radius: 10px; box-shadow: 0 2px 10px rgba(0,0,0,0.05); }
        .card-header { padding: 15px 20px; border-bottom: 1px solid #eee; font-weight: bold; }
        .card-body { padding: 20px; }
        table { width: 100%; border-collapse: collapse; }
        table th, table td { padding: 12px 15px; text-align: left; border-bottom: 1px solid #eee; }
        table th { background: #f8f9fa; font-weight: 500; color: #666; }
        .pagination { display: flex; justify-content: center; gap: 5px; margin-top: 20px; }
        .pagination a, .pagination span { padding: 8px 12px; border: 1px solid #ddd; border-radius: 4px; text-decoration: none; color: #666; }
        .pagination a:hover { background: #667eea; color: white; border-color: #667eea; }
        .pagination .active { background: #667eea; color: white; border-color: #667eea; }
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
        <a href="index.php">概览</a>
        <a href="devices.php">设备管理</a>
        <a href="appeals.php">申诉管理</a>
        <a href="logs.php" class="active">操作日志</a>
    </div>

    <div class="container">
        <div class="card">
            <div class="card-header">操作日志 (共 <?php echo $total; ?> 条)</div>
            <div class="card-body">
                <table>
                    <thead>
                        <tr>
                            <th>时间</th>
                            <th>操作员</th>
                            <th>操作</th>
                            <th>目标</th>
                            <th>详情</th>
                            <th>IP</th>
                        </tr>
                    </thead>
                    <tbody>
                        <?php foreach ($logs as $log): ?>
                        <tr>
                            <td><?php echo $log['created_at']; ?></td>
                            <td><?php echo htmlspecialchars($log['nickname'] ?: $log['username']); ?></td>
                            <td><?php echo $actionMap[$log['action']] ?? $log['action']; ?></td>
                            <td>
                                <?php if ($log['target_type']): ?>
                                    <?php echo $log['target_type']; ?>:<?php echo $log['target_id']; ?>
                                <?php else: ?>
                                    -
                                <?php endif; ?>
                            </td>
                            <td><?php echo htmlspecialchars($log['detail'] ?: '-'); ?></td>
                            <td><?php echo htmlspecialchars($log['ip_address']); ?></td>
                        </tr>
                        <?php endforeach; ?>
                        <?php if (empty($logs)): ?>
                        <tr>
                            <td colspan="6" style="text-align:center; color:#888; padding:40px;">暂无操作日志</td>
                        </tr>
                        <?php endif; ?>
                    </tbody>
                </table>

                <?php if ($totalPages > 1): ?>
                <div class="pagination">
                    <?php if ($page > 1): ?>
                        <a href="?page=<?php echo $page - 1; ?>">上一页</a>
                    <?php endif; ?>

                    <?php for ($i = max(1, $page - 2); $i <= min($totalPages, $page + 2); $i++): ?>
                        <?php if ($i == $page): ?>
                            <span class="active"><?php echo $i; ?></span>
                        <?php else: ?>
                            <a href="?page=<?php echo $i; ?>"><?php echo $i; ?></a>
                        <?php endif; ?>
                    <?php endfor; ?>

                    <?php if ($page < $totalPages): ?>
                        <a href="?page=<?php echo $page + 1; ?>">下一页</a>
                    <?php endif; ?>
                </div>
                <?php endif; ?>
            </div>
        </div>
    </div>
</body>
</html>
