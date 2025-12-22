<?php
/**
 * DevAssistant 管理后台 - 申诉管理
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

// 状态过滤
$status = $_GET['status'] ?? '';

$where = [];
$params = [];

if ($status !== '') {
    $where[] = 'a.status = ?';
    $params[] = intval($status);
}

$whereClause = $where ? 'WHERE ' . implode(' AND ', $where) : '';

// 获取总数
$stmt = $db->prepare("SELECT COUNT(*) as total FROM da_appeals a {$whereClause}");
$stmt->execute($params);
$total = $stmt->fetch()['total'];
$totalPages = ceil($total / $perPage);

// 获取列表
$stmt = $db->prepare("
    SELECT a.*, d.hostname, d.username, d.mac_address
    FROM da_appeals a
    LEFT JOIN da_devices d ON a.device_id = d.device_id
    {$whereClause}
    ORDER BY a.status ASC, a.created_at DESC
    LIMIT {$offset}, {$perPage}
");
$stmt->execute($params);
$appeals = $stmt->fetchAll();

$statusMap = [0 => '待处理', 1 => '已通过', 2 => '已拒绝'];
$statusClass = [0 => 'warning', 1 => 'success', 2 => 'danger'];
?>
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>申诉管理 - DevAssistant 管理后台</title>
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
        .filter-tabs { display: flex; gap: 10px; }
        .filter-tabs a { padding: 6px 15px; border-radius: 20px; text-decoration: none; color: #666; background: #f0f0f0; }
        .filter-tabs a.active { background: #667eea; color: white; }
        table { width: 100%; border-collapse: collapse; }
        table th, table td { padding: 12px 15px; text-align: left; border-bottom: 1px solid #eee; }
        table th { background: #f8f9fa; font-weight: 500; color: #666; }
        .badge { display: inline-block; padding: 4px 10px; border-radius: 20px; font-size: 12px; }
        .badge-success { background: #d1fae5; color: #059669; }
        .badge-danger { background: #fee2e2; color: #dc2626; }
        .badge-warning { background: #fef3c7; color: #d97706; }
        .btn { padding: 6px 12px; border: none; border-radius: 4px; cursor: pointer; font-size: 13px; margin-right: 5px; }
        .btn-success { background: #10b981; color: white; }
        .btn-danger { background: #ef4444; color: white; }
        .reason-cell { max-width: 300px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
        .reason-cell:hover { white-space: normal; overflow: visible; }
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
        <a href="devices.php">设备管理</a>
        <a href="appeals.php" class="active">申诉管理</a>
        <a href="logs.php">操作日志</a>
    </div>

    <div class="container">
        <div class="card">
            <div class="card-header">
                <strong>申诉列表 (共 <?php echo $total; ?> 条)</strong>
                <div class="filter-tabs">
                    <a href="?status=" class="<?php echo $status === '' ? 'active' : ''; ?>">全部</a>
                    <a href="?status=0" class="<?php echo $status === '0' ? 'active' : ''; ?>">待处理</a>
                    <a href="?status=1" class="<?php echo $status === '1' ? 'active' : ''; ?>">已通过</a>
                    <a href="?status=2" class="<?php echo $status === '2' ? 'active' : ''; ?>">已拒绝</a>
                </div>
            </div>
            <div class="card-body">
                <table>
                    <thead>
                        <tr>
                            <th>ID</th>
                            <th>设备信息</th>
                            <th>联系方式</th>
                            <th>申诉理由</th>
                            <th>提交时间</th>
                            <th>状态</th>
                            <th>操作</th>
                        </tr>
                    </thead>
                    <tbody>
                        <?php foreach ($appeals as $appeal): ?>
                        <tr>
                            <td><?php echo $appeal['id']; ?></td>
                            <td>
                                <div><?php echo htmlspecialchars($appeal['hostname'] ?: '-'); ?></div>
                                <small style="color:#888;"><?php echo htmlspecialchars($appeal['mac_address'] ?? '-'); ?></small>
                            </td>
                            <td><?php echo htmlspecialchars($appeal['contact']); ?></td>
                            <td class="reason-cell" title="<?php echo htmlspecialchars($appeal['reason']); ?>">
                                <?php echo htmlspecialchars($appeal['reason']); ?>
                            </td>
                            <td><?php echo $appeal['created_at']; ?></td>
                            <td>
                                <span class="badge badge-<?php echo $statusClass[$appeal['status']]; ?>">
                                    <?php echo $statusMap[$appeal['status']]; ?>
                                </span>
                            </td>
                            <td>
                                <?php if ($appeal['status'] == 0): ?>
                                    <button class="btn btn-success" onclick="approveAppeal(<?php echo $appeal['id']; ?>)">通过</button>
                                    <button class="btn btn-danger" onclick="rejectAppeal(<?php echo $appeal['id']; ?>)">拒绝</button>
                                <?php else: ?>
                                    <span style="color:#888;">已处理</span>
                                <?php endif; ?>
                            </td>
                        </tr>
                        <?php endforeach; ?>
                        <?php if (empty($appeals)): ?>
                        <tr>
                            <td colspan="7" style="text-align:center; color:#888; padding:40px;">暂无申诉记录</td>
                        </tr>
                        <?php endif; ?>
                    </tbody>
                </table>

                <?php if ($totalPages > 1): ?>
                <div class="pagination">
                    <?php if ($page > 1): ?>
                        <a href="?page=<?php echo $page - 1; ?>&status=<?php echo $status; ?>">上一页</a>
                    <?php endif; ?>

                    <?php for ($i = max(1, $page - 2); $i <= min($totalPages, $page + 2); $i++): ?>
                        <?php if ($i == $page): ?>
                            <span class="active"><?php echo $i; ?></span>
                        <?php else: ?>
                            <a href="?page=<?php echo $i; ?>&status=<?php echo $status; ?>"><?php echo $i; ?></a>
                        <?php endif; ?>
                    <?php endfor; ?>

                    <?php if ($page < $totalPages): ?>
                        <a href="?page=<?php echo $page + 1; ?>&status=<?php echo $status; ?>">下一页</a>
                    <?php endif; ?>
                </div>
                <?php endif; ?>
            </div>
        </div>
    </div>

    <!-- 拒绝弹窗 -->
    <div class="modal" id="rejectModal">
        <div class="modal-content">
            <h3>拒绝申诉</h3>
            <textarea id="rejectReply" placeholder="请输入拒绝理由..."></textarea>
            <div class="modal-actions">
                <button class="btn" onclick="closeModal()">取消</button>
                <button class="btn btn-danger" onclick="confirmReject()">确认拒绝</button>
            </div>
        </div>
    </div>

    <script>
        let currentAppealId = null;

        function approveAppeal(id) {
            if (!confirm('确定通过该申诉并解禁设备吗？')) return;

            fetch('api.php?action=approve_appeal', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ appeal_id: id, reply: '申诉通过，已解禁' })
            })
            .then(r => r.json())
            .then(data => {
                if (data.code === 200) {
                    alert('操作成功');
                    location.reload();
                } else {
                    alert(data.message);
                }
            });
        }

        function rejectAppeal(id) {
            currentAppealId = id;
            document.getElementById('rejectModal').classList.add('show');
        }

        function closeModal() {
            document.getElementById('rejectModal').classList.remove('show');
            currentAppealId = null;
        }

        function confirmReject() {
            const reply = document.getElementById('rejectReply').value.trim();
            if (!reply) {
                alert('请输入拒绝理由');
                return;
            }

            fetch('api.php?action=reject_appeal', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ appeal_id: currentAppealId, reply: reply })
            })
            .then(r => r.json())
            .then(data => {
                if (data.code === 200) {
                    alert('操作成功');
                    location.reload();
                } else {
                    alert(data.message);
                }
            });
        }
    </script>
</body>
</html>
