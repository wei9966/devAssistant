<?php
/**
 * DevAssistant 管理后台 - 设备管理
 */

define('DA_ROOT', dirname(__DIR__));
require_once DA_ROOT . '/includes/common.php';
require_once DA_ROOT . '/includes/auth.php';

requireAdmin();

$db = getDB();

// 分页参数
$page = max(1, intval($_GET['page'] ?? 1));
$perPage = 20;
$offset = ($page - 1) * $perPage;

// 搜索参数
$search = trim($_GET['search'] ?? '');
$status = $_GET['status'] ?? '';

// 构建查询
$where = [];
$params = [];

if ($search) {
    $where[] = '(hostname LIKE ? OR username LIKE ? OR mac_address LIKE ?)';
    $params[] = "%{$search}%";
    $params[] = "%{$search}%";
    $params[] = "%{$search}%";
}

if ($status !== '') {
    $where[] = 'status = ?';
    $params[] = intval($status);
}

$whereClause = $where ? 'WHERE ' . implode(' AND ', $where) : '';

// 获取总数
$stmt = $db->prepare("SELECT COUNT(*) as total FROM da_devices {$whereClause}");
$stmt->execute($params);
$total = $stmt->fetch()['total'];
$totalPages = ceil($total / $perPage);

// 获取列表
$stmt = $db->prepare("SELECT * FROM da_devices {$whereClause} ORDER BY last_seen DESC LIMIT {$offset}, {$perPage}");
$stmt->execute($params);
$devices = $stmt->fetchAll();
?>
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>设备管理 - DevAssistant 管理后台</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #f5f7fa; min-height: 100vh; }
        .header { background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 15px 30px; display: flex; justify-content: space-between; align-items: center; }
        .header h1 { font-size: 20px; }
        .header a { color: white; text-decoration: none; }
        .nav { background: white; padding: 0 30px; border-bottom: 1px solid #e0e0e0; }
        .nav a { display: inline-block; padding: 15px 20px; color: #666; text-decoration: none; border-bottom: 2px solid transparent; }
        .nav a:hover, .nav a.active { color: #667eea; border-bottom-color: #667eea; }
        .container { max-width: 1400px; margin: 0 auto; padding: 30px; }
        .card { background: white; border-radius: 10px; box-shadow: 0 2px 10px rgba(0,0,0,0.05); }
        .card-header { padding: 15px 20px; border-bottom: 1px solid #eee; display: flex; justify-content: space-between; align-items: center; }
        .card-body { padding: 20px; }
        .search-form { display: flex; gap: 10px; }
        .search-form input, .search-form select {
            padding: 8px 12px; border: 1px solid #ddd; border-radius: 5px; font-size: 14px;
        }
        .search-form button { padding: 8px 20px; background: #667eea; color: white; border: none; border-radius: 5px; cursor: pointer; }
        table { width: 100%; border-collapse: collapse; }
        table th, table td { padding: 12px 15px; text-align: left; border-bottom: 1px solid #eee; }
        table th { background: #f8f9fa; font-weight: 500; color: #666; }
        .badge { display: inline-block; padding: 4px 10px; border-radius: 20px; font-size: 12px; }
        .badge-success { background: #d1fae5; color: #059669; }
        .badge-danger { background: #fee2e2; color: #dc2626; }
        .btn { padding: 6px 12px; border: none; border-radius: 4px; cursor: pointer; font-size: 13px; }
        .btn-danger { background: #ef4444; color: white; }
        .btn-success { background: #10b981; color: white; }
        .btn-info { background: #3b82f6; color: white; }
        .pagination { display: flex; justify-content: center; gap: 5px; margin-top: 20px; }
        .pagination a, .pagination span { padding: 8px 12px; border: 1px solid #ddd; border-radius: 4px; text-decoration: none; color: #666; }
        .pagination a:hover { background: #667eea; color: white; border-color: #667eea; }
        .pagination .active { background: #667eea; color: white; border-color: #667eea; }
        .modal { display: none; position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); align-items: center; justify-content: center; }
        .modal.show { display: flex; }
        .modal-content { background: white; padding: 30px; border-radius: 10px; width: 90%; max-width: 500px; }
        .modal-content h3 { margin-bottom: 20px; }
        .modal-content textarea { width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 5px; min-height: 100px; margin-bottom: 15px; }
        .modal-actions { display: flex; gap: 10px; justify-content: flex-end; }
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
        <a href="devices.php" class="active">设备管理</a>
        <a href="appeals.php">申诉管理</a>
        <a href="logs.php">操作日志</a>
    </div>

    <div class="container">
        <div class="card">
            <div class="card-header">
                <strong>设备列表 (共 <?php echo $total; ?> 条)</strong>
                <form class="search-form" method="GET">
                    <input type="text" name="search" placeholder="搜索设备名/用户名/MAC" value="<?php echo htmlspecialchars($search); ?>">
                    <select name="status">
                        <option value="">全部状态</option>
                        <option value="1" <?php echo $status === '1' ? 'selected' : ''; ?>>正常</option>
                        <option value="0" <?php echo $status === '0' ? 'selected' : ''; ?>>已禁用</option>
                    </select>
                    <button type="submit">搜索</button>
                </form>
            </div>
            <div class="card-body">
                <table>
                    <thead>
                        <tr>
                            <th>ID</th>
                            <th>计算机名</th>
                            <th>用户名</th>
                            <th>MAC地址</th>
                            <th>系统</th>
                            <th>版本</th>
                            <th>上报次数</th>
                            <th>最后活跃</th>
                            <th>状态</th>
                            <th>操作</th>
                        </tr>
                    </thead>
                    <tbody>
                        <?php foreach ($devices as $device): ?>
                        <tr>
                            <td><?php echo $device['id']; ?></td>
                            <td><?php echo htmlspecialchars($device['hostname'] ?: '-'); ?></td>
                            <td><?php echo htmlspecialchars($device['username'] ?: '-'); ?></td>
                            <td><code><?php echo htmlspecialchars($device['mac_address']); ?></code></td>
                            <td><?php echo htmlspecialchars($device['os_info'] ?: '-'); ?></td>
                            <td><?php echo htmlspecialchars($device['app_version'] ?: '-'); ?></td>
                            <td><?php echo $device['total_reports']; ?></td>
                            <td><?php echo $device['last_seen']; ?></td>
                            <td>
                                <?php if ($device['status'] == 1): ?>
                                    <span class="badge badge-success">正常</span>
                                <?php else: ?>
                                    <span class="badge badge-danger">禁用</span>
                                <?php endif; ?>
                            </td>
                            <td>
                                <button class="btn btn-info" onclick="viewDetail('<?php echo $device['device_id']; ?>')">详情</button>
                                <?php if ($device['status'] == 1): ?>
                                    <button class="btn btn-danger" onclick="banDevice('<?php echo $device['device_id']; ?>')">禁用</button>
                                <?php else: ?>
                                    <button class="btn btn-success" onclick="unbanDevice('<?php echo $device['device_id']; ?>')">解禁</button>
                                <?php endif; ?>
                            </td>
                        </tr>
                        <?php endforeach; ?>
                    </tbody>
                </table>

                <?php if ($totalPages > 1): ?>
                <div class="pagination">
                    <?php if ($page > 1): ?>
                        <a href="?page=<?php echo $page - 1; ?>&search=<?php echo urlencode($search); ?>&status=<?php echo $status; ?>">上一页</a>
                    <?php endif; ?>

                    <?php for ($i = max(1, $page - 2); $i <= min($totalPages, $page + 2); $i++): ?>
                        <?php if ($i == $page): ?>
                            <span class="active"><?php echo $i; ?></span>
                        <?php else: ?>
                            <a href="?page=<?php echo $i; ?>&search=<?php echo urlencode($search); ?>&status=<?php echo $status; ?>"><?php echo $i; ?></a>
                        <?php endif; ?>
                    <?php endfor; ?>

                    <?php if ($page < $totalPages): ?>
                        <a href="?page=<?php echo $page + 1; ?>&search=<?php echo urlencode($search); ?>&status=<?php echo $status; ?>">下一页</a>
                    <?php endif; ?>
                </div>
                <?php endif; ?>
            </div>
        </div>
    </div>

    <!-- 禁用弹窗 -->
    <div class="modal" id="banModal">
        <div class="modal-content">
            <h3>禁用设备</h3>
            <textarea id="banReason" placeholder="请输入禁用原因..."></textarea>
            <div class="modal-actions">
                <button class="btn" onclick="closeModal()">取消</button>
                <button class="btn btn-danger" onclick="confirmBan()">确认禁用</button>
            </div>
        </div>
    </div>

    <script>
        let currentDeviceId = null;

        function banDevice(deviceId) {
            currentDeviceId = deviceId;
            document.getElementById('banModal').classList.add('show');
        }

        function closeModal() {
            document.getElementById('banModal').classList.remove('show');
            currentDeviceId = null;
        }

        function confirmBan() {
            const reason = document.getElementById('banReason').value.trim();
            if (!reason) {
                alert('请输入禁用原因');
                return;
            }

            fetch('api.php?action=ban', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ device_id: currentDeviceId, reason: reason })
            })
            .then(r => r.json())
            .then(data => {
                if (data.code === 200) {
                    alert('禁用成功');
                    location.reload();
                } else {
                    alert(data.message);
                }
            });
        }

        function unbanDevice(deviceId) {
            if (!confirm('确定要解禁该设备吗？')) return;

            fetch('api.php?action=unban', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ device_id: deviceId })
            })
            .then(r => r.json())
            .then(data => {
                if (data.code === 200) {
                    alert('解禁成功');
                    location.reload();
                } else {
                    alert(data.message);
                }
            });
        }

        function viewDetail(deviceId) {
            window.open('device_detail.php?id=' + deviceId, '_blank');
        }
    </script>
</body>
</html>
