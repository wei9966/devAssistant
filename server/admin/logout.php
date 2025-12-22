<?php
/**
 * DevAssistant 管理后台 - 退出登录
 */

define('DA_ROOT', dirname(__DIR__));
require_once DA_ROOT . '/includes/common.php';
require_once DA_ROOT . '/includes/auth.php';

adminLogout();

header('Location: login.php');
exit;
