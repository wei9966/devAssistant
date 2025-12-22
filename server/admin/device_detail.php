<?php
/**
 * DevAssistant 管理后台 - 设备详情
 */

define('DA_ROOT', dirname(__DIR__));
require_once DA_ROOT . '/includes/common.php';
require_once DA_ROOT . '/includes/auth.php';

requireAdmin();

$db = getDB();
$deviceId = $_GET['id'] ?? '';

if (empty($deviceId)) {
    header('Location: devices.php');
    exit;
}

// 获取设备信息
$stmt = $db->prepare('SELECT * FROM da_devices WHERE device_id = ?');
$stmt->execute([$deviceId]);
$device = $stmt->fetch();

if (!$device) {
    header('Location: devices.php');
    exit;
}

// 获取心跳记录
$stmt = $db->prepare('
    SELECT * FROM da_heartbeats
    WHERE device_id = ?
    ORDER BY report_time DESC
    LIMIT 50
');
$stmt->execute([$deviceId]);
$heartbeats = $stmt->fetchAll();

// 获取申诉记录
$stmt = $db->prepare('
    SELECT * FROM da_appeals
    WHERE device_id = ?
    ORDER BY created_at DESC
');
$stmt->execute([$deviceId]);
$appeals = $stmt->fetchAll();

$statusMap = [0 => '待处理', 1 => '已通过', 2 => '已拒绝'];
?>
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>设备详情 - DevAssistant 管理后台</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #f5f7fa; min-height: 100vh; }
        .header { background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 15px 30px; display: flex; justify-content: space-between; align-items: center; }
        .header h1 { font-size: 20px; }
        .header a { color: white; text-decoration: none; }
        .container { max-width: 1200px; margin: 0 auto; padding: 30px; }
        .back-link { display: inline-block; margin-bottom: 20px; color: #667eea; text-decoration: none; }
        .card { background: white; border-radius: 10px; box-shadow: 0 2px 10px rgba(0,0,0,0.05); margin-bottom: 20px; }
        .card-header { padding: 15px 20px; border-bottom: 1px solid #eee; font-weight: bold; display: flex; justify-content: space-between; align-items: center; }
        .card-body { padding: 20px; }
        .info-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 20px; }
        .info-item { padding: 15px; background: #f8f9fa; border-radius: 8px; }
        .info-item label { display: block; color: #888; font-size: 13px; margin-bottom: 5px; }
        .info-item .value { font-size: 16px; color: #333; }
        .badge { display: inline-block; padding: 4px 10px; border-radius: 20px; font-size: 12px; }
        .badge-success { background: #d1fae5; color: #059669; }
        .badge-danger { background: #fee2e2; color: #dc2626; }
        table { width: 100%; border-collapse: collapse; }
        table th, table td { padding: 10px 15px; text-align: left; border-bottom: 1px solid #eee; font-size: 14px; }
        table th { background: #f8f9fa; font-weight: 500; color: #666; }
        .btn { padding: 8px 16px; border: none; border-radius: 5px; cursor: pointer; font-size: 14px; }
        .btn-danger { background: #ef4444; color: white; }
        .btn-success { background: #10b981; color: white; }
        textarea { width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 5px; min-height: 80px; }
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

    <div class="container">
        <a href="devices.php" class="back-link">&larr; 返回设备列表</a>

        <div class="card">
            <div class="card-header">
                <span>设备信息</span>
                <?php if ($device['status'] == 1): ?>
                    <button class="btn btn-danger" onclick="banDevice()">禁用此设备</button>
                <?php else: ?>
                    <button class="btn btn-success" onclick="unbanDevice()">解禁此设备</button>
                <?php endif; ?>
            </div>
            <div class="card-body">
                <div class="info-grid">
                    <div class="info-item">
                        <label>设备ID</label>
                        <div class="value"><code><?php echo $device['device_id']; ?></code></div>
                    </div>
                    <div class="info-item">
                        <label>MAC地址</label>
                        <div class="value"><?php echo htmlspecialchars($device['mac_address']); ?></div>
                    </div>
                    <div class="info-item">
                        <label>计算机名</label>
                        <div class="value"><?php echo htmlspecialchars($device['hostname'] ?: '-'); ?></div>
                    </div>
                    <div class="info-item">
                        <label>系统用户名</label>
                        <div class="value"><?php echo htmlspecialchars($device['username'] ?: '-'); ?></div>
                    </div>
                    <div class="info-item">
                        <label>操作系统</label>
                        <div class="value"><?php echo htmlspecialchars($device['os_info'] ?: '-'); ?></div>
                    </div>
                    <div class="info-item">
                        <label>软件版本</label>
                        <div class="value"><?php echo htmlspecialchars($device['app_version'] ?: '-'); ?></div>
                    </div>
                    <div class="info-item">
                        <label>首次上报</label>
                        <div class="value"><?php echo $device['first_seen']; ?></div>
                    </div>
                    <div class="info-item">
                        <label>最后活跃</label>
                        <div class="value"><?php echo $device['last_seen']; ?></div>
                    </div>
                    <div class="info-item">
                        <label>累计上报</label>
                        <div class="value"><?php echo $device['total_reports']; ?> 次</div>
                    </div>
                    <div class="info-item">
                        <label>状态</label>
                        <div class="value">
                            <?php if ($device['status'] == 1): ?>
                                <span class="badge badge-success">正常</span>
                            <?php else: ?>
                                <span class="badge badge-danger">禁用</span>
                                <?php if ($device['ban_reason']): ?>
                                    <br><small style="color:#888;">原因: <?php echo htmlspecialchars($device['ban_reason']); ?></small>
                                <?php endif; ?>
                            <?php endif; ?>
                        </div>
                    </div>
                </div>

                <div style="margin-top: 20px;">
                    <label style="display:block; margin-bottom:8px; color:#666;">备注</label>
                    <textarea id="remark" placeholder="输入备注信息..."><?php echo htmlspecialchars($device['remark']); ?></textarea>
                    <button class="btn btn-success" style="margin-top:10px;" onclick="saveRemark()">保存备注</button>
                </div>
            </div>
        </div>

        <div class="card">
            <div class="card-header">心跳记录 (最近50条)</div>
            <div class="card-body">
                <table>
                    <thead>
                        <tr>
                            <th>上报时间</th>
                            <th>公网IP</th>
                            <th>本地IP</th>
                            <th>版本</th>
                        </tr>
                    </thead>
                    <tbody>
                        <?php foreach ($heartbeats as $hb): ?>
                        <tr>
                            <td><?php echo $hb['report_time']; ?></td>
                            <td><?php echo htmlspecialchars($hb['ip_address']); ?></td>
                            <td><?php echo htmlspecialchars($hb['local_ip'] ?: '-'); ?></td>
                            <td><?php echo htmlspecialchars($hb['app_version'] ?: '-'); ?></td>
                        </tr>
                        <?php endforeach; ?>
                        <?php if (empty($heartbeats)): ?>
                        <tr>
                            <td colspan="4" style="text-align:center; color:#888;">暂无记录</td>
                        </tr>
                        <?php endif; ?>
                    </tbody>
                </table>
            </div>
        </div>

        <?php if (!empty($appeals)): ?>
        <div class="card">
            <div class="card-header">申诉记录</div>
            <div class="card-body">
                <table>
                    <thead>
                        <tr>
                            <th>提交时间</th>
                            <th>联系方式</th>
                            <th>申诉理由</th>
                            <th>状态</th>
                            <th>回复</th>
                        </tr>
                    </thead>
                    <tbody>
                        <?php foreach ($appeals as $appeal): ?>
                        <tr>
                            <td><?php echo $appeal['created_at']; ?></td>
                            <td><?php echo htmlspecialchars($appeal['contact']); ?></td>
                            <td><?php echo htmlspecialchars($appeal['reason']); ?></td>
                            <td><?php echo $statusMap[$appeal['status']]; ?></td>
                            <td><?php echo htmlspecialchars($appeal['admin_reply'] ?: '-'); ?></td>
                        </tr>
                        <?php endforeach; ?>
                    </tbody>
                </table>
            </div>
        </div>
        <?php endif; ?>
    </div>

    <script>
        const deviceId = '<?php echo $device['device_id']; ?>';

        function banDevice() {
            const reason = prompt('请输入禁用原因:');
            if (!reason) return;

            fetch('api.php?action=ban', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ device_id: deviceId, reason: reason })
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

        function unbanDevice() {
            if (!confirm('确定要解禁此设备吗?')) return;

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

        function saveRemark() {
            const remark = document.getElementById('remark').value;

            fetch('api.php?action=update_remark', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ device_id: deviceId, remark: remark })
            })
            .then(r => r.json())
            .then(data => {
                if (data.code === 200) {
                    alert('保存成功');
                } else {
                    alert(data.message);
                }
            });
        }
    </script>
</body>
</html>
