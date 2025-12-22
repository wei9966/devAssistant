DevAssistant 用户手册 - 部署说明
====================================

部署目录结构
------------
deploy/
├── index.html      <- 主页面文件
├── images/         <- 图片目录（需要手动复制）
│   └── *.png
└── README.txt      <- 本说明文件


部署步骤
--------

【步骤1】复制图片到 images 目录

Windows CMD:
    mkdir images
    copy ..\..\pic\introduce\*.png images\

Windows PowerShell:
    New-Item -ItemType Directory -Force -Path images
    Copy-Item ..\..\pic\introduce\*.png -Destination images\

Linux/Mac:
    mkdir -p images
    cp ../../pic/introduce/*.png images/


【步骤2】上传到服务器

方式A - FTP/SFTP上传:
    将整个 deploy 文件夹上传到服务器的网站目录

方式B - SCP命令上传:
    scp -r deploy/* user@your-server:/var/www/html/devassistant-docs/

方式C - 宝塔面板:
    1. 打开宝塔面板 -> 文件
    2. 进入网站根目录
    3. 上传 deploy 文件夹内容


【步骤3】访问测试

假设部署到 /devassistant-docs/ 目录：
    http://your-domain.com/devassistant-docs/

或直接部署到根目录：
    http://your-domain.com/


Nginx 配置示例（可选）
----------------------
server {
    listen 80;
    server_name docs.your-domain.com;
    root /var/www/html/devassistant-docs;
    index index.html;

    location / {
        try_files $uri $uri/ =404;
    }
}


嵌入软件使用
------------
如果要在 DevAssistant 软件内嵌入此文档：

1. 在线版本：
   直接通过 WebView 打开部署后的 URL

2. 离线版本：
   将 deploy 文件夹放入软件资源目录，使用 file:// 协议打开


注意事项
--------
1. 图片文件较大，首次加载可能需要一些时间
2. 建议开启服务器的 gzip 压缩
3. 可配置 CDN 加速图片加载
4. 图片已启用懒加载，不会一次性加载全部


联系方式
--------
如有问题请联系开发者

====================================
