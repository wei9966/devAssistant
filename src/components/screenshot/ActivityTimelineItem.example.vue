<template>
  <div class="example-container">
    <h2>ActivityTimelineItem 使用示例</h2>

    <!-- 示例 1: 编码活动 -->
    <section class="example-section">
      <h3>编码活动</h3>
      <ActivityTimelineItem
        :activity="codingActivity"
        @preview="handlePreview"
      />
    </section>

    <!-- 示例 2: 浏览活动 -->
    <section class="example-section">
      <h3>浏览活动</h3>
      <ActivityTimelineItem
        :activity="browsingActivity"
        @preview="handlePreview"
      />
    </section>

    <!-- 示例 3: 文档处理活动 -->
    <section class="example-section">
      <h3>文档处理活动</h3>
      <ActivityTimelineItem
        :activity="documentActivity"
        @preview="handlePreview"
      />
    </section>

    <!-- 截图预览弹窗 -->
    <ScreenshotDetailModal
      v-if="selectedScreenshot"
      v-model:show="showPreview"
      :screenshot="selectedScreenshot"
    />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import ActivityTimelineItem, { type ActivityGroup } from './ActivityTimelineItem.vue'
import ScreenshotDetailModal from './ScreenshotDetailModal.vue'
import type { ScreenshotRecord } from '@/api/screenshotApi'

const showPreview = ref(false)
const selectedScreenshot = ref<ScreenshotRecord | null>(null)

// 示例数据
const codingActivity: ActivityGroup = {
  activityType: 'coding',
  description: '开发用户认证模块，实现登录、注册、密码重置功能。包括前端表单验证、后端API接口开发、数据库设计等工作。',
  screenshots: [
    {
      id: 1,
      capturedAt: '2025-12-04T09:30:00Z',
      appName: 'Visual Studio Code',
      windowTitle: 'auth.service.ts - DevAssistant',
      activityType: 'coding',
      description: '编写用户认证服务',
      keyContent: 'async login(credentials)',
      screenshotPath: '/screenshots/2025-12-04/001.png',
      processingTimeMs: 150
    },
    {
      id: 2,
      capturedAt: '2025-12-04T09:45:00Z',
      appName: 'Visual Studio Code',
      windowTitle: 'auth.controller.ts - DevAssistant',
      activityType: 'coding',
      description: '实现认证控制器',
      keyContent: 'POST /api/auth/login',
      screenshotPath: '/screenshots/2025-12-04/002.png',
      processingTimeMs: 120
    },
    {
      id: 3,
      capturedAt: '2025-12-04T10:00:00Z',
      appName: 'Postman',
      windowTitle: 'Auth API Testing',
      activityType: 'coding',
      description: '测试认证接口',
      keyContent: 'POST login endpoint',
      screenshotPath: '/screenshots/2025-12-04/003.png',
      processingTimeMs: 90
    }
  ]
}

const browsingActivity: ActivityGroup = {
  activityType: 'browsing',
  description: '查阅Vue 3最新文档，学习Composition API的高级用法，研究性能优化技巧和最佳实践。',
  screenshots: [
    {
      id: 4,
      capturedAt: '2025-12-04T11:00:00Z',
      appName: 'Google Chrome',
      windowTitle: 'Composition API | Vue.js',
      activityType: 'browsing',
      description: '学习Vue 3组合式API',
      keyContent: 'ref, reactive, computed',
      screenshotPath: '/screenshots/2025-12-04/004.png',
      processingTimeMs: 100
    },
    {
      id: 5,
      capturedAt: '2025-12-04T11:15:00Z',
      appName: 'Google Chrome',
      windowTitle: 'Performance | Vue.js',
      activityType: 'browsing',
      description: '研究性能优化',
      keyContent: 'lazy loading, code splitting',
      screenshotPath: '/screenshots/2025-12-04/005.png',
      processingTimeMs: 110
    }
  ]
}

const documentActivity: ActivityGroup = {
  activityType: 'document',
  description: '编写项目技术文档，包括系统架构设计、API文档、部署说明等内容。',
  screenshots: [
    {
      id: 6,
      capturedAt: '2025-12-04T14:00:00Z',
      appName: 'Microsoft Word',
      windowTitle: '系统架构设计.docx',
      activityType: 'document',
      description: '编写系统架构文档',
      keyContent: '微服务架构',
      screenshotPath: '/screenshots/2025-12-04/006.png',
      processingTimeMs: 80
    },
    {
      id: 7,
      capturedAt: '2025-12-04T14:30:00Z',
      appName: 'Microsoft Word',
      windowTitle: 'API文档.docx',
      activityType: 'document',
      description: '编写API接口文档',
      keyContent: 'RESTful API',
      screenshotPath: '/screenshots/2025-12-04/007.png',
      processingTimeMs: 95
    },
    {
      id: 8,
      capturedAt: '2025-12-04T15:00:00Z',
      appName: 'Typora',
      windowTitle: 'README.md',
      activityType: 'document',
      description: '更新README文档',
      keyContent: '安装说明、使用指南',
      screenshotPath: '/screenshots/2025-12-04/008.png',
      processingTimeMs: 70
    },
    {
      id: 9,
      capturedAt: '2025-12-04T15:30:00Z',
      appName: 'Typora',
      windowTitle: 'DEPLOYMENT.md',
      activityType: 'document',
      description: '编写部署文档',
      keyContent: 'Docker部署',
      screenshotPath: '/screenshots/2025-12-04/009.png',
      processingTimeMs: 85
    }
  ]
}

function handlePreview(screenshot: ScreenshotRecord) {
  selectedScreenshot.value = screenshot
  showPreview.value = true
}
</script>

<style scoped>
.example-container {
  padding: 24px;
  max-width: 1200px;
  margin: 0 auto;
}

h2 {
  font-size: 24px;
  font-weight: 600;
  color: rgb(226, 232, 240);
  margin-bottom: 32px;
}

.example-section {
  margin-bottom: 32px;
}

.example-section h3 {
  font-size: 16px;
  font-weight: 500;
  color: rgb(203, 213, 225);
  margin-bottom: 16px;
}
</style>
