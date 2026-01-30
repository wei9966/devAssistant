@echo off
chcp 65001 >nul
echo ========================================
echo   DevAssistant 发布版本打包脚本
echo ========================================
echo.

:: 设置签名环境变量
set TAURI_SIGNING_PRIVATE_KEY=dW50cnVzdGVkIGNvbW1lbnQ6IHJzaWduIGVuY3J5cHRlZCBzZWNyZXQga2V5ClJXUlRZMEl5VVpFeHNmUjdqblJBZHVTNEFGZzdMaGdGUllEZjlqWE5MakdORlpOWUYzUUFBQkFBQUFBQUFBQUFBQUlBQUFBQTgrSlVFSU9xZzNzZVVkK1VzenF1MS9vK2lNbVgzOXdnd3Z4Ym01NE9CTHlWbWMrQ25teVVGc013TkY4MVYwMVY3cmt0STk5QzhqV1Q3WjJrR2lPSkQ4dEwzU1lWajJwNXRHSXFEalQzeEc4dTA1QnBYRjZQUWtwTTN6NG4vQnhIYVBRVkJ2RzE5Q1k9Cg==
set TAURI_SIGNING_PRIVATE_KEY_PASSWORD=996649855

echo [1/3] 环境变量已设置
echo.

echo [2/3] 开始打包...
echo.

:: 执行打包命令
call npm run tauri:build

echo.
echo ========================================
if %ERRORLEVEL% EQU 0 (
    echo   打包成功！
    echo.
    echo   安装包位置:
    echo   src-tauri\target\release\bundle\nsis\
    echo.
    echo   需要上传的文件:
    echo   - DevAssistant_x.x.x_x64-setup.nsis.zip
    echo   - DevAssistant_x.x.x_x64-setup.nsis.zip.sig (签名)
) else (
    echo   打包失败，请检查错误信息
)
echo ========================================
echo.
pause
