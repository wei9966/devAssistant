<?php
/**
 * DevAssistant 管理后台 - 登录页面
 */

// 开启错误显示（调试用，正式环境请关闭）
ini_set('display_errors', 1);
ini_set('display_startup_errors', 1);
error_reporting(E_ALL);

// 调试：显示PHP版本和扩展信息
echo "<!-- PHP Version: " . PHP_VERSION . " -->\n";
echo "<!-- PDO Extension: " . (extension_loaded('pdo') ? 'YES' : 'NO') . " -->\n";
echo "<!-- PDO MySQL Extension: " . (extension_loaded('pdo_mysql') ? 'YES' : 'NO') . " -->\n";

define('DA_ROOT', dirname(__DIR__));
echo "<!-- DA_ROOT: " . DA_ROOT . " -->\n";

// 检查配置文件是否存在
if (!file_exists(DA_ROOT . '/config/database.php')) {
    die('错误: 找不到数据库配置文件，路径: ' . DA_ROOT . '/config/database.php');
}

// 检查文件是否存在
if (!file_exists(DA_ROOT . '/includes/common.php')) {
    die('错误: 找不到 common.php 文件，路径: ' . DA_ROOT . '/includes/common.php');
}
if (!file_exists(DA_ROOT . '/includes/auth.php')) {
    die('错误: 找不到 auth.php 文件，路径: ' . DA_ROOT . '/includes/auth.php');
}

require_once DA_ROOT . '/includes/common.php';
require_once DA_ROOT . '/includes/auth.php';

// 处理登录请求
$error = '';
if ($_SERVER['REQUEST_METHOD'] === 'POST') {
    $username = trim($_POST['username'] ?? '');
    $password = trim($_POST['password'] ?? '');

    if (empty($username) || empty($password)) {
        $error = '请输入用户名和密码';
    } else {
        $result = adminLogin($username, $password);
        if ($result['success']) {
            header('Location: index.php');
            exit;
        } else {
            $error = $result['message'];
        }
    }
}

// 如果已登录，跳转到首页
if (isAdminLoggedIn()) {
    header('Location: index.php');
    exit;
}
?>
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>登录 - DevAssistant 管理后台</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
        }
        .login-box {
            background: white;
            padding: 40px;
            border-radius: 10px;
            box-shadow: 0 15px 35px rgba(0,0,0,0.2);
            width: 100%;
            max-width: 400px;
        }
        .login-box h1 {
            text-align: center;
            color: #333;
            margin-bottom: 30px;
            font-size: 24px;
        }
        .form-group {
            margin-bottom: 20px;
        }
        .form-group label {
            display: block;
            margin-bottom: 8px;
            color: #555;
            font-size: 14px;
        }
        .form-group input {
            width: 100%;
            padding: 12px 15px;
            border: 1px solid #ddd;
            border-radius: 5px;
            font-size: 14px;
            transition: border-color 0.3s;
        }
        .form-group input:focus {
            outline: none;
            border-color: #667eea;
        }
        .error {
            background: #fee;
            color: #c00;
            padding: 10px 15px;
            border-radius: 5px;
            margin-bottom: 20px;
            font-size: 14px;
        }
        .btn {
            width: 100%;
            padding: 12px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            border: none;
            border-radius: 5px;
            font-size: 16px;
            cursor: pointer;
            transition: opacity 0.3s;
        }
        .btn:hover {
            opacity: 0.9;
        }
    </style>
</head>
<body>
    <div class="login-box">
        <h1>DevAssistant 管理后台</h1>
        <?php if ($error): ?>
            <div class="error"><?php echo htmlspecialchars($error); ?></div>
        <?php endif; ?>
        <form method="POST">
            <div class="form-group">
                <label for="username">用户名</label>
                <input type="text" id="username" name="username" required autofocus>
            </div>
            <div class="form-group">
                <label for="password">密码</label>
                <input type="password" id="password" name="password" required>
            </div>
            <button type="submit" class="btn">登 录</button>
        </form>
    </div>
</body>
</html>
