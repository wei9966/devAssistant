@echo off
chcp 65001 >nul
setlocal enabledelayedexpansion

echo ========================================
echo   DevAssistant 一键打包脚本
echo ========================================
echo.

:: 检查 Node.js
where node >nul 2>nul
if %errorlevel% neq 0 (
    echo [错误] 未找到 Node.js，请先安装 Node.js
    pause
    exit /b 1
)

:: 检查 Rust
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo [错误] 未找到 Rust/Cargo，请先安装 Rust
    pause
    exit /b 1
)

:: 进入项目目录
cd /d "%~dp0"

echo [1/4] 安装前端依赖...
call npm install
if %errorlevel% neq 0 (
    echo [错误] npm install 失败
    pause
    exit /b 1
)

echo.
echo [2/4] 构建前端...
call npm run build
if %errorlevel% neq 0 (
    echo [错误] 前端构建失败
    pause
    exit /b 1
)

echo.
echo [3/4] 构建 Tauri 应用 (Release)...
call npx tauri build
if %errorlevel% neq 0 (
    echo [警告] Tauri build 可能部分失败，检查输出...
)

echo.
echo [4/4] 尝试生成 NSIS 安装包...
call npx tauri build --bundles nsis
if %errorlevel% neq 0 (
    echo [警告] NSIS 打包可能失败，但 exe 文件应该已生成
)

echo.
echo ========================================
echo   打包完成!
echo ========================================
echo.
echo 输出文件位置:
echo   可执行文件: src-tauri\target\release\dev-assistant.exe
echo   安装包目录: src-tauri\target\release\bundle\nsis\
echo.

:: 打开输出目录
if exist "src-tauri\target\release\dev-assistant.exe" (
    echo 正在打开输出目录...
    explorer "src-tauri\target\release"
)

pause
