# DevAssistant 一键打包脚本 (PowerShell)
# 用法: 右键 -> 使用 PowerShell 运行

param(
    [switch]$SkipInstall,      # 跳过 npm install
    [switch]$NsisOnly,         # 仅生成 NSIS 安装包
    [switch]$ExeOnly,          # 仅生成 exe 文件
    [switch]$Clean             # 清理构建缓存后重新构建
)

$ErrorActionPreference = "Stop"
$Host.UI.RawUI.WindowTitle = "DevAssistant 打包"

function Write-Header {
    param([string]$text)
    Write-Host ""
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host "  $text" -ForegroundColor Cyan
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host ""
}

function Write-Step {
    param([string]$step, [string]$text)
    Write-Host "[$step] $text" -ForegroundColor Yellow
}

function Write-Success {
    param([string]$text)
    Write-Host "[成功] $text" -ForegroundColor Green
}

function Write-Error {
    param([string]$text)
    Write-Host "[错误] $text" -ForegroundColor Red
}

# 开始
Write-Header "DevAssistant 一键打包脚本"

# 切换到脚本所在目录
Set-Location $PSScriptRoot

# 检查环境
Write-Step "0/4" "检查构建环境..."

$nodeVersion = node --version 2>$null
if (-not $nodeVersion) {
    Write-Error "未找到 Node.js，请先安装"
    Read-Host "按回车退出"
    exit 1
}
Write-Host "  Node.js: $nodeVersion" -ForegroundColor Gray

$cargoVersion = cargo --version 2>$null
if (-not $cargoVersion) {
    Write-Error "未找到 Rust/Cargo，请先安装"
    Read-Host "按回车退出"
    exit 1
}
Write-Host "  Cargo: $cargoVersion" -ForegroundColor Gray

# 清理缓存（如果指定）
if ($Clean) {
    Write-Step "0.5/4" "清理构建缓存..."
    if (Test-Path "src-tauri\target") {
        Remove-Item -Recurse -Force "src-tauri\target"
    }
    if (Test-Path "dist") {
        Remove-Item -Recurse -Force "dist"
    }
    if (Test-Path "node_modules") {
        Remove-Item -Recurse -Force "node_modules"
        $SkipInstall = $false
    }
    Write-Success "缓存已清理"
}

# 安装依赖
if (-not $SkipInstall) {
    Write-Step "1/4" "安装前端依赖..."
    npm install
    if ($LASTEXITCODE -ne 0) {
        Write-Error "npm install 失败"
        Read-Host "按回车退出"
        exit 1
    }
    Write-Success "依赖安装完成"
} else {
    Write-Host "[跳过] npm install" -ForegroundColor Gray
}

# 构建前端
Write-Step "2/4" "构建前端..."
npm run build
if ($LASTEXITCODE -ne 0) {
    Write-Error "前端构建失败"
    Read-Host "按回车退出"
    exit 1
}
Write-Success "前端构建完成"

# 构建 Tauri
Write-Step "3/4" "构建 Tauri 应用 (Release)..."
$startTime = Get-Date

if ($NsisOnly) {
    Write-Host "  仅生成 NSIS 安装包..." -ForegroundColor Gray
    npx tauri build --bundles nsis
} elseif ($ExeOnly) {
    Write-Host "  仅生成可执行文件..." -ForegroundColor Gray
    npx tauri build --bundles none
} else {
    npx tauri build
}

$buildResult = $LASTEXITCODE
$endTime = Get-Date
$duration = $endTime - $startTime

if ($buildResult -ne 0) {
    Write-Host "[警告] Tauri 构建可能部分失败" -ForegroundColor Yellow
}

Write-Host "  构建耗时: $($duration.Minutes)分$($duration.Seconds)秒" -ForegroundColor Gray

# 尝试 NSIS 打包
if (-not $ExeOnly -and -not $NsisOnly) {
    Write-Step "4/4" "尝试生成 NSIS 安装包..."
    npx tauri build --bundles nsis
    if ($LASTEXITCODE -ne 0) {
        Write-Host "[警告] NSIS 打包可能失败，但 exe 文件应该已生成" -ForegroundColor Yellow
    }
}

# 输出结果
Write-Header "打包完成!"

$exePath = "src-tauri\target\release\dev-assistant.exe"
$nsisDir = "src-tauri\target\release\bundle\nsis"

if (Test-Path $exePath) {
    $exeSize = [math]::Round((Get-Item $exePath).Length / 1MB, 2)
    Write-Host "可执行文件:" -ForegroundColor White
    Write-Host "  路径: $exePath" -ForegroundColor Gray
    Write-Host "  大小: ${exeSize} MB" -ForegroundColor Gray
    Write-Host ""
}

if (Test-Path $nsisDir) {
    $installers = Get-ChildItem $nsisDir -Filter "*.exe" 2>$null
    if ($installers) {
        Write-Host "安装包:" -ForegroundColor White
        foreach ($installer in $installers) {
            $size = [math]::Round($installer.Length / 1MB, 2)
            Write-Host "  $($installer.Name) (${size} MB)" -ForegroundColor Gray
        }
        Write-Host ""
    }
}

# 打开输出目录
if (Test-Path $exePath) {
    $openFolder = Read-Host "是否打开输出目录? (Y/N)"
    if ($openFolder -eq "Y" -or $openFolder -eq "y") {
        explorer "src-tauri\target\release"
    }
}

Write-Host ""
Write-Host "按任意键退出..." -ForegroundColor Gray
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
