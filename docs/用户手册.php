<?php
/**
 * DevAssistant 用户手册 - PHP版本
 * 可部署到Web服务器用于线上浏览
 * 也可以嵌入软件内通过WebView打开
 */

// 配置
$config = [
    'app_name' => 'DevAssistant',
    'version' => 'v1.0.20',
    'image_base_path' => '../pic/introduce/',
];

// 图片列表
$screenshots = [
    'task-board' => ['file' => '微信图片_20251222133607_258_1.png', 'title' => '任务看板主界面', 'desc' => '四象限任务分类展示'],
    'task-new' => ['file' => '微信图片_20251222133616_259_1.png', 'title' => '新建任务弹窗', 'desc' => '支持AI智能分类'],
    'task-float' => ['file' => '微信图片_20251222133622_260_1.png', 'title' => '任务悬浮窗', 'desc' => '精简的任务列表视图'],
    'calendar-month' => ['file' => '微信图片_20251222133736_261_1.png', 'title' => '任务日历', 'desc' => '月视图'],
    'calendar-week' => ['file' => '微信图片_20251222133737_262_1.png', 'title' => '任务日历', 'desc' => '周视图'],
    'ai-predict' => ['file' => '微信图片_20251222133738_263_1.png', 'title' => 'AI预测任务', 'desc' => '智能识别潜在待办事项'],
    'task-menu' => ['file' => '微信图片_20251222133739_264_1.png', 'title' => '任务操作菜单', 'desc' => ''],
    'milestone' => ['file' => '微信图片_20251222133739_265_1.png', 'title' => '添加里程碑功能', 'desc' => ''],
    'pomodoro' => ['file' => '微信图片_20251222133740_266_1.png', 'title' => '专注时钟', 'desc' => '番茄工作法'],
    'sql-history' => ['file' => '微信图片_20251222133743_267_1.png', 'title' => 'SQL执行记录页面', 'desc' => ''],
    'quick-input' => ['file' => '微信图片_20251222133748_268_1.png', 'title' => '快速任务/SQL输入框', 'desc' => ''],
    'sql-search' => ['file' => '微信图片_20251222133753_269_1.png', 'title' => 'SQL搜索和预览', 'desc' => ''],
    'worklog-daily' => ['file' => '微信图片_20251222133815_270_1.png', 'title' => '工作日志', 'desc' => '日报视图'],
    'worklog-weekly' => ['file' => '微信图片_20251222133817_271_1.png', 'title' => '工作日志', 'desc' => '周报视图'],
    'worklog-plan' => ['file' => '微信图片_20251222133818_272_1.png', 'title' => '工作日志', 'desc' => '周计划视图'],
    'launcher' => ['file' => '微信图片_20251222133820_273_1.png', 'title' => '应用启动器', 'desc' => '分类管理常用应用'],
    'launcher-add' => ['file' => '微信图片_20251222133827_274_1.png', 'title' => '添加应用弹窗', 'desc' => ''],
    'launcher-search' => ['file' => '微信图片_20251222133831_275_1.png', 'title' => '应用快捷搜索', 'desc' => ''],
    'report' => ['file' => '微信图片_20251222133836_276_1.png', 'title' => '报表中心', 'desc' => '任务报表'],
    'notification' => ['file' => '微信图片_20251222133843_277_1.png', 'title' => '通知中心', 'desc' => '集中管理消息'],
    'toolbox' => ['file' => '微信图片_20251222133847_278_1.png', 'title' => '工具箱', 'desc' => '实用开发工具集'],
    'text-converter' => ['file' => '微信图片_20251222133902_280_1.png', 'title' => '文本转换器', 'desc' => '支持多种文本处理操作'],
    'settings-general' => ['file' => '微信图片_20251222133906_281_1.png', 'title' => '通用设置', 'desc' => '基础配置项'],
    'settings-ai' => ['file' => '微信图片_20251222133924_282_1.png', 'title' => '集成设置', 'desc' => 'AI智能助手配置'],
    'settings-ai-log' => ['file' => '微信图片_20251222133928_283_1.png', 'title' => 'AI日志', 'desc' => '调用统计和日志'],
    'settings-prompt' => ['file' => '微信图片_20251222133938_284_1.png', 'title' => '提示词管理', 'desc' => '自定义AI提示词'],
    'settings-notify' => ['file' => '微信图片_20251222133941_285_1.png', 'title' => '通知设置', 'desc' => '定时任务配置'],
    'settings-screen' => ['file' => '微信图片_20251222133944_286_1.png', 'title' => '屏幕上下文', 'desc' => '采集设置'],
    'settings-vlm' => ['file' => '微信图片_20251222134005_287_1.png', 'title' => 'VLM配置', 'desc' => '视觉语言模型设置'],
    'about' => ['file' => '微信图片_20251222134013_288_1.png', 'title' => '关于', 'desc' => '版本信息和更新'],
];

// 辅助函数：生成截图HTML
function renderScreenshot($key, $screenshots, $basePath) {
    $img = $screenshots[$key];
    $caption = $img['title'] . ($img['desc'] ? ' - ' . $img['desc'] : '');
    return <<<HTML
    <div class="screenshot" onclick="openLightbox(this)">
        <img src="{$basePath}{$img['file']}" alt="{$img['title']}" loading="lazy">
        <div class="screenshot-caption">{$caption}</div>
    </div>
HTML;
}
?>
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title><?php echo $config['app_name']; ?> 用户手册</title>
    <style>
        :root {
            --bg-primary: #1a1a2e;
            --bg-secondary: #16213e;
            --bg-card: #1f2940;
            --text-primary: #e8e8e8;
            --text-secondary: #a0a0a0;
            --accent-primary: #7c3aed;
            --accent-secondary: #4f46e5;
            --border-color: #2d3748;
            --success-color: #10b981;
            --warning-color: #f59e0b;
        }

        * { margin: 0; padding: 0; box-sizing: border-box; }

        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
            background: var(--bg-primary);
            color: var(--text-primary);
            line-height: 1.6;
        }

        .navbar {
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            height: 60px;
            background: var(--bg-secondary);
            border-bottom: 1px solid var(--border-color);
            display: flex;
            align-items: center;
            padding: 0 24px;
            z-index: 1000;
        }

        .navbar-brand {
            font-size: 20px;
            font-weight: 600;
            color: var(--accent-primary);
            display: flex;
            align-items: center;
            gap: 10px;
        }

        .sidebar {
            position: fixed;
            top: 60px;
            left: 0;
            width: 280px;
            height: calc(100vh - 60px);
            background: var(--bg-secondary);
            border-right: 1px solid var(--border-color);
            overflow-y: auto;
            padding: 20px 0;
        }

        .sidebar-section { margin-bottom: 8px; }

        .sidebar-title {
            padding: 8px 24px;
            font-size: 12px;
            text-transform: uppercase;
            color: var(--text-secondary);
            font-weight: 600;
        }

        .sidebar-link {
            display: block;
            padding: 10px 24px;
            color: var(--text-primary);
            text-decoration: none;
            transition: all 0.2s;
            border-left: 3px solid transparent;
        }

        .sidebar-link:hover, .sidebar-link.active {
            background: var(--bg-card);
            border-left-color: var(--accent-primary);
        }

        .sidebar-sublink { padding-left: 40px; font-size: 14px; color: var(--text-secondary); }

        .main-content {
            margin-left: 280px;
            margin-top: 60px;
            padding: 40px;
            max-width: 1000px;
        }

        h1 { font-size: 36px; margin-bottom: 16px; }
        h2 { font-size: 28px; margin: 48px 0 24px; padding-bottom: 12px; border-bottom: 2px solid var(--accent-primary); }
        h3 { font-size: 22px; margin: 32px 0 16px; }
        h4 { font-size: 18px; margin: 24px 0 12px; color: var(--accent-primary); }
        p { margin-bottom: 16px; color: var(--text-secondary); }
        .lead { font-size: 18px; color: var(--text-primary); margin-bottom: 24px; }

        .hero {
            background: linear-gradient(135deg, var(--accent-primary), var(--accent-secondary));
            padding: 40px;
            border-radius: 16px;
            margin-bottom: 40px;
            text-align: center;
        }

        .hero h1 { color: white; margin-bottom: 8px; }
        .hero .subtitle { color: rgba(255,255,255,0.8); font-size: 18px; margin-bottom: 16px; }
        .hero .version {
            display: inline-block;
            background: rgba(255,255,255,0.2);
            padding: 4px 16px;
            border-radius: 20px;
            font-size: 14px;
            color: white;
        }

        .tech-stack {
            margin-top: 20px;
            display: flex;
            justify-content: center;
            gap: 12px;
            flex-wrap: wrap;
        }

        .tech-badge {
            background: rgba(255,255,255,0.15);
            padding: 6px 14px;
            border-radius: 6px;
            font-size: 13px;
            color: white;
        }

        .features-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
            gap: 20px;
            margin: 24px 0;
        }

        .feature-card {
            background: var(--bg-card);
            padding: 24px;
            border-radius: 12px;
            border: 1px solid var(--border-color);
            transition: transform 0.2s, box-shadow 0.2s;
        }

        .feature-card:hover {
            transform: translateY(-4px);
            box-shadow: 0 8px 24px rgba(0,0,0,0.3);
        }

        .feature-icon {
            width: 48px;
            height: 48px;
            background: var(--accent-primary);
            border-radius: 12px;
            display: flex;
            align-items: center;
            justify-content: center;
            margin-bottom: 16px;
            font-size: 24px;
        }

        .feature-card h4 { margin: 0 0 8px; color: var(--text-primary); }
        .feature-card p { margin: 0; font-size: 14px; }

        .screenshot {
            margin: 24px 0;
            border-radius: 12px;
            overflow: hidden;
            border: 1px solid var(--border-color);
            box-shadow: 0 4px 16px rgba(0,0,0,0.2);
            cursor: pointer;
        }

        .screenshot img { width: 100%; display: block; }

        .screenshot-caption {
            background: var(--bg-card);
            padding: 12px 16px;
            font-size: 14px;
            color: var(--text-secondary);
            border-top: 1px solid var(--border-color);
        }

        .feature-list {
            background: var(--bg-card);
            padding: 24px;
            border-radius: 12px;
            margin: 24px 0;
            border: 1px solid var(--border-color);
        }

        .feature-list h4 { margin-top: 0; margin-bottom: 16px; }
        .feature-list ul { list-style: none; padding: 0; }
        .feature-list li {
            padding: 8px 0;
            padding-left: 24px;
            position: relative;
            color: var(--text-secondary);
        }
        .feature-list li::before {
            content: '✓';
            position: absolute;
            left: 0;
            color: var(--success-color);
            font-weight: bold;
        }

        .table-wrapper { overflow-x: auto; margin: 24px 0; }
        table {
            width: 100%;
            border-collapse: collapse;
            background: var(--bg-card);
            border-radius: 12px;
            overflow: hidden;
        }
        th, td { padding: 14px 20px; text-align: left; border-bottom: 1px solid var(--border-color); }
        th { background: var(--bg-secondary); color: var(--text-primary); font-weight: 600; }
        td { color: var(--text-secondary); }
        tr:last-child td { border-bottom: none; }
        code {
            background: var(--bg-secondary);
            padding: 2px 8px;
            border-radius: 4px;
            font-family: 'Consolas', 'Monaco', monospace;
            color: var(--accent-primary);
        }

        .faq-item {
            background: var(--bg-card);
            margin: 16px 0;
            border-radius: 12px;
            border: 1px solid var(--border-color);
            overflow: hidden;
        }

        .faq-question {
            padding: 20px 24px;
            font-weight: 600;
            color: var(--text-primary);
        }

        .faq-answer { padding: 0 24px 20px; color: var(--text-secondary); }

        .back-to-top {
            position: fixed;
            bottom: 30px;
            right: 30px;
            width: 48px;
            height: 48px;
            background: var(--accent-primary);
            border-radius: 50%;
            display: flex;
            align-items: center;
            justify-content: center;
            color: white;
            text-decoration: none;
            box-shadow: 0 4px 16px rgba(124, 58, 237, 0.4);
            transition: transform 0.2s;
        }

        .back-to-top:hover { transform: translateY(-4px); }

        .footer {
            margin-top: 60px;
            padding: 40px;
            background: var(--bg-secondary);
            border-top: 1px solid var(--border-color);
            text-align: center;
            color: var(--text-secondary);
        }

        @media (max-width: 768px) {
            .sidebar { display: none; }
            .main-content { margin-left: 0; padding: 20px; }
            .hero { padding: 24px; }
            .hero h1 { font-size: 28px; }
        }

        ::-webkit-scrollbar { width: 8px; }
        ::-webkit-scrollbar-track { background: var(--bg-primary); }
        ::-webkit-scrollbar-thumb { background: var(--border-color); border-radius: 4px; }
        ::-webkit-scrollbar-thumb:hover { background: var(--accent-primary); }

        .lightbox {
            display: none;
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: rgba(0,0,0,0.9);
            z-index: 2000;
            align-items: center;
            justify-content: center;
            cursor: zoom-out;
        }
        .lightbox.active { display: flex; }
        .lightbox img { max-width: 90%; max-height: 90%; border-radius: 8px; }
    </style>
</head>
<body>
    <nav class="navbar">
        <div class="navbar-brand">
            <svg viewBox="0 0 24 24" fill="currentColor" width="32" height="32">
                <rect x="3" y="3" width="7" height="7" rx="1"/>
                <rect x="14" y="3" width="7" height="7" rx="1"/>
                <rect x="3" y="14" width="7" height="7" rx="1"/>
                <rect x="14" y="14" width="7" height="7" rx="1"/>
            </svg>
            <?php echo $config['app_name']; ?> 用户手册
        </div>
    </nav>

    <aside class="sidebar">
        <div class="sidebar-section">
            <a href="#intro" class="sidebar-link active">软件简介</a>
        </div>
        <div class="sidebar-section">
            <div class="sidebar-title">核心功能</div>
            <a href="#task" class="sidebar-link">任务管理</a>
            <a href="#pomodoro" class="sidebar-link">专注时钟</a>
            <a href="#sql" class="sidebar-link">SQL管理</a>
            <a href="#worklog" class="sidebar-link">工作日志</a>
            <a href="#launcher" class="sidebar-link">应用启动器</a>
            <a href="#report" class="sidebar-link">报表中心</a>
            <a href="#notification" class="sidebar-link">通知中心</a>
            <a href="#toolbox" class="sidebar-link">工具箱</a>
        </div>
        <div class="sidebar-section">
            <div class="sidebar-title">系统配置</div>
            <a href="#settings" class="sidebar-link">系统设置</a>
        </div>
        <div class="sidebar-section">
            <a href="#shortcuts" class="sidebar-link">快捷键</a>
            <a href="#faq" class="sidebar-link">常见问题</a>
        </div>
    </aside>

    <main class="main-content">
        <div class="hero" id="intro">
            <h1><?php echo $config['app_name']; ?></h1>
            <p class="subtitle">个人开发效率工具</p>
            <span class="version"><?php echo $config['version']; ?></span>
            <div class="tech-stack">
                <span class="tech-badge">Tauri</span>
                <span class="tech-badge">Rust</span>
                <span class="tech-badge">Vue3</span>
                <span class="tech-badge">TypeScript</span>
                <span class="tech-badge">Naive UI</span>
            </div>
        </div>

        <p class="lead">DevAssistant 是一款专为开发者打造的个人效率工具，集成了任务管理、番茄钟、SQL历史记录、工作日志、应用启动器等多种实用功能。</p>

        <div class="features-grid">
            <div class="feature-card">
                <div class="feature-icon">📋</div>
                <h4>智能任务管理</h4>
                <p>支持四象限分类、AI智能分类、任务预测</p>
            </div>
            <div class="feature-card">
                <div class="feature-icon">⏱️</div>
                <h4>专注时钟</h4>
                <p>番茄工作法，帮助保持专注</p>
            </div>
            <div class="feature-card">
                <div class="feature-icon">💾</div>
                <h4>SQL历史管理</h4>
                <p>自动记录SQL执行历史，方便查询复用</p>
            </div>
            <div class="feature-card">
                <div class="feature-icon">📝</div>
                <h4>工作日志</h4>
                <p>日报、周报、周计划一站式管理</p>
            </div>
            <div class="feature-card">
                <div class="feature-icon">🚀</div>
                <h4>应用启动器</h4>
                <p>快速启动常用应用，支持工作流</p>
            </div>
            <div class="feature-card">
                <div class="feature-icon">🤖</div>
                <h4>AI助手</h4>
                <p>支持自定义AI服务，提供智能建议</p>
            </div>
        </div>

        <h2 id="task">任务管理</h2>
        <p>任务看板是软件的核心功能，采用四象限法则对任务进行分类管理。</p>
        <?php echo renderScreenshot('task-board', $screenshots, $config['image_base_path']); ?>

        <div class="feature-list">
            <h4>📌 功能说明</h4>
            <ul>
                <li>左侧导航栏可快速切换各功能模块</li>
                <li>任务按照四象限分类展示</li>
                <li>支持拖拽调整任务所属象限</li>
                <li>底部状态栏显示系统CPU和内存使用情况</li>
                <li>支持快捷键 <code>Ctrl + K</code> 快速搜索</li>
            </ul>
        </div>

        <h3>新建任务</h3>
        <?php echo renderScreenshot('task-new', $screenshots, $config['image_base_path']); ?>

        <h3>任务悬浮窗</h3>
        <?php echo renderScreenshot('task-float', $screenshots, $config['image_base_path']); ?>

        <h3>任务日历</h3>
        <?php echo renderScreenshot('calendar-month', $screenshots, $config['image_base_path']); ?>
        <?php echo renderScreenshot('calendar-week', $screenshots, $config['image_base_path']); ?>

        <h3>AI预测任务</h3>
        <?php echo renderScreenshot('ai-predict', $screenshots, $config['image_base_path']); ?>

        <h3>任务操作</h3>
        <?php echo renderScreenshot('task-menu', $screenshots, $config['image_base_path']); ?>
        <?php echo renderScreenshot('milestone', $screenshots, $config['image_base_path']); ?>

        <h2 id="pomodoro">专注时钟</h2>
        <p>专注时钟基于番茄工作法，帮助您保持工作专注。</p>
        <?php echo renderScreenshot('pomodoro', $screenshots, $config['image_base_path']); ?>

        <h2 id="sql">SQL管理</h2>
        <p>自动记录您执行过的SQL语句，方便后续查询和复用。</p>
        <?php echo renderScreenshot('sql-history', $screenshots, $config['image_base_path']); ?>
        <?php echo renderScreenshot('quick-input', $screenshots, $config['image_base_path']); ?>
        <?php echo renderScreenshot('sql-search', $screenshots, $config['image_base_path']); ?>

        <h2 id="worklog">工作日志</h2>
        <p>工作日志帮助您记录每日工作内容，生成周报和周计划。</p>
        <?php echo renderScreenshot('worklog-daily', $screenshots, $config['image_base_path']); ?>
        <?php echo renderScreenshot('worklog-weekly', $screenshots, $config['image_base_path']); ?>
        <?php echo renderScreenshot('worklog-plan', $screenshots, $config['image_base_path']); ?>

        <h2 id="launcher">应用启动器</h2>
        <p>快速启动常用应用程序，提高工作效率。</p>
        <?php echo renderScreenshot('launcher', $screenshots, $config['image_base_path']); ?>
        <?php echo renderScreenshot('launcher-add', $screenshots, $config['image_base_path']); ?>
        <?php echo renderScreenshot('launcher-search', $screenshots, $config['image_base_path']); ?>

        <h2 id="report">报表中心</h2>
        <?php echo renderScreenshot('report', $screenshots, $config['image_base_path']); ?>

        <h2 id="notification">通知中心</h2>
        <?php echo renderScreenshot('notification', $screenshots, $config['image_base_path']); ?>

        <h2 id="toolbox">工具箱</h2>
        <?php echo renderScreenshot('toolbox', $screenshots, $config['image_base_path']); ?>
        <h3>文本转换器</h3>
        <?php echo renderScreenshot('text-converter', $screenshots, $config['image_base_path']); ?>

        <h2 id="settings">系统设置</h2>
        <h3>通用设置</h3>
        <?php echo renderScreenshot('settings-general', $screenshots, $config['image_base_path']); ?>
        <h3>AI集成</h3>
        <?php echo renderScreenshot('settings-ai', $screenshots, $config['image_base_path']); ?>
        <?php echo renderScreenshot('settings-ai-log', $screenshots, $config['image_base_path']); ?>
        <?php echo renderScreenshot('settings-prompt', $screenshots, $config['image_base_path']); ?>
        <h3>通知设置</h3>
        <?php echo renderScreenshot('settings-notify', $screenshots, $config['image_base_path']); ?>
        <h3>屏幕上下文</h3>
        <?php echo renderScreenshot('settings-screen', $screenshots, $config['image_base_path']); ?>
        <?php echo renderScreenshot('settings-vlm', $screenshots, $config['image_base_path']); ?>
        <h3>关于</h3>
        <?php echo renderScreenshot('about', $screenshots, $config['image_base_path']); ?>

        <h2 id="shortcuts">快捷键说明</h2>
        <div class="table-wrapper">
            <table>
                <thead><tr><th>快捷键</th><th>功能</th></tr></thead>
                <tbody>
                    <tr><td><code>Ctrl + K</code></td><td>全局搜索</td></tr>
                    <tr><td><code>Ctrl + N</code></td><td>新建任务</td></tr>
                    <tr><td><code>Ctrl + Space</code></td><td>唤起应用启动器搜索</td></tr>
                    <tr><td><code>Esc</code></td><td>关闭弹窗</td></tr>
                </tbody>
            </table>
        </div>

        <h2 id="faq">常见问题</h2>
        <div class="faq-item">
            <div class="faq-question">Q: 如何配置AI功能？</div>
            <div class="faq-answer">A: 进入设置 > 集成，启用AI功能并配置API Key和Base URL。支持任何OpenAI兼容的API服务。</div>
        </div>
        <div class="faq-item">
            <div class="faq-question">Q: 屏幕截图会占用很多存储空间吗？</div>
            <div class="faq-answer">A: 可以在设置 > 屏幕上下文中配置数据保留天数，系统会自动清理过期数据。</div>
        </div>
        <div class="faq-item">
            <div class="faq-question">Q: 如何导出工作日志？</div>
            <div class="faq-answer">A: 在工作日志页面，点击导出按钮可将日报/周报导出为常用格式。</div>
        </div>

        <div class="footer">
            <p><strong>感谢使用 <?php echo $config['app_name']; ?>！</strong></p>
            <p>如有问题或建议，欢迎反馈。</p>
            <p style="margin-top: 16px; font-size: 12px;">生成时间：<?php echo date('Y-m-d H:i:s'); ?></p>
        </div>
    </main>

    <a href="#intro" class="back-to-top">↑</a>

    <div class="lightbox" id="lightbox" onclick="closeLightbox()">
        <img src="" alt="" id="lightbox-img">
    </div>

    <script>
        function openLightbox(element) {
            const img = element.querySelector('img');
            const lightbox = document.getElementById('lightbox');
            const lightboxImg = document.getElementById('lightbox-img');
            lightboxImg.src = img.src;
            lightbox.classList.add('active');
        }

        function closeLightbox() {
            document.getElementById('lightbox').classList.remove('active');
        }

        document.addEventListener('keydown', (e) => {
            if (e.key === 'Escape') closeLightbox();
        });

        // 侧边栏激活
        const sidebarLinks = document.querySelectorAll('.sidebar-link');
        window.addEventListener('scroll', () => {
            const sections = document.querySelectorAll('h2[id], h3[id]');
            let current = '';
            sections.forEach(section => {
                if (scrollY >= section.offsetTop - 100) {
                    current = section.getAttribute('id');
                }
            });
            sidebarLinks.forEach(link => {
                link.classList.remove('active');
                if (link.getAttribute('href') === '#' + current) {
                    link.classList.add('active');
                }
            });
        });
    </script>
</body>
</html>
