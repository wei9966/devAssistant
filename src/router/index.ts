// Vue Router
import { createRouter, createWebHistory } from 'vue-router'
import type { RouteRecordRaw } from 'vue-router'

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    redirect: '/task-board'
  },
  {
    path: '/task-board',
    name: 'task-board',
    component: () => import('../views/TaskBoard.vue'),
    meta: {
      title: '任务看板'
    }
  },
  {
    path: '/sql-history',
    name: 'sql-history',
    component: () => import('../views/SqlHistory.vue'),
    meta: {
      title: 'SQL历史'
    }
  },
  {
    path: '/work-log',
    name: 'work-log',
    component: () => import('../views/WorkLog.vue'),
    meta: {
      title: '工作日志'
    }
  },
  {
    path: '/app-launcher',
    name: 'app-launcher',
    component: () => import('../views/AppLauncher.vue'),
    meta: {
      title: '应用启动器'
    }
  },
  {
    path: '/screenshot-gallery',
    name: 'screenshot-gallery',
    component: () => import('../views/ScreenshotGallery.vue'),
    meta: {
      title: '截图回顾'
    }
  },
  {
    path: '/report-center',
    name: 'report-center',
    component: () => import('../views/ReportCenter.vue'),
    meta: {
      title: '报表中心'
    }
  },
  {
    path: '/notification-center',
    name: 'notification-center',
    component: () => import('../views/NotificationCenter.vue'),
    meta: {
      title: '通知中心'
    }
  },
  {
    path: '/settings',
    name: 'settings',
    component: () => import('../views/Settings.vue'),
    meta: {
      title: '设置'
    }
  }
]

export const router = createRouter({
  history: createWebHistory(),
  routes
})
