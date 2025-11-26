<template>
  <n-modal
    v-model:show="showModal"
    preset="card"
    title="启动器设置"
    :style="{ width: '600px' }"
    :bordered="false"
    :segmented="{ content: 'soft', footer: 'soft' }"
  >
    <n-spin :show="loading">
      <div class="settings-content">
        <!-- 允许的文件类型 -->
        <div class="settings-section">
          <div class="section-header">
            <h3>允许添加的文件类型</h3>
            <p class="section-desc">
              配置可以添加到启动器的文件后缀。默认只允许 .exe 和 .lnk 快捷方式。
            </p>
          </div>

          <div class="extensions-list">
            <n-tag
              v-for="ext in settings.allowedExtensions"
              :key="ext"
              :closable="!isDefaultExtension(ext)"
              :type="isDefaultExtension(ext) ? 'info' : 'default'"
              @close="handleRemoveExtension(ext)"
              class="extension-tag"
            >
              .{{ ext }}
            </n-tag>
          </div>

          <div class="add-extension">
            <n-input
              v-model:value="newExtension"
              placeholder="输入文件后缀(如: pdf, rdp, url)"
              @keyup.enter="handleAddExtension"
              :style="{ width: '300px' }"
            >
              <template #prefix>.</template>
            </n-input>
            <n-button type="primary" @click="handleAddExtension" :disabled="!newExtension.trim()">
              添加
            </n-button>
          </div>

          <!-- 常用文件类型快捷添加 -->
          <div class="common-extensions">
            <span class="label">常用类型:</span>
            <n-button
              v-for="ext in commonExtensions"
              :key="ext.value"
              size="small"
              :disabled="settings.allowedExtensions.includes(ext.value)"
              @click="handleQuickAdd(ext.value)"
              class="quick-add-btn"
            >
              {{ ext.label }}
            </n-button>
          </div>
        </div>

        <!-- 帮助说明 -->
        <n-alert type="info" :bordered="false" class="help-alert">
          <template #header>
            <n-icon size="18"><InformationCircleOutline /></n-icon>
            <span>说明</span>
          </template>
          <ul>
            <li><strong>.exe</strong> 和 <strong>.lnk</strong> 是默认文件类型,无法删除</li>
            <li>扫描应用时,只会扫描允许列表中的文件类型</li>
            <li>添加 <strong>.rdp</strong> 可以管理远程桌面连接</li>
            <li>添加 <strong>.url</strong> 可以管理网页链接</li>
            <li>也可以添加任意文件类型,如 .pdf, .docx 等</li>
          </ul>
        </n-alert>
      </div>
    </n-spin>

    <template #footer>
      <div class="footer-actions">
        <n-button @click="handleReset" :disabled="loading">
          重置为默认
        </n-button>
        <div class="right-actions">
          <n-button @click="handleCancel" :disabled="loading">取消</n-button>
          <n-button type="primary" @click="handleSave" :loading="loading">
            保存
          </n-button>
        </div>
      </div>
    </template>
  </n-modal>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import {
  NModal,
  NButton,
  NInput,
  NTag,
  NSpin,
  NAlert,
  NIcon,
  useMessage,
} from 'naive-ui';
import { InformationCircleOutline } from '@vicons/ionicons5';
import type { AppLauncherSettings } from '@/types/appLauncher';
import { invoke } from '@tauri-apps/api/core';

interface Props {
  show: boolean;
}

interface Emits {
  (e: 'update:show', value: boolean): void;
  (e: 'saved'): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const message = useMessage();

const showModal = ref(false);
const loading = ref(false);
// 初始值会在对话框打开时被后端数据覆盖
const settings = ref<AppLauncherSettings>({
  allowedExtensions: ['exe', 'lnk'],
});
const newExtension = ref('');

// 默认扩展名(不可删除)
const defaultExtensions = ['exe', 'lnk'];

// 常用文件类型
const commonExtensions = [
  { label: '远程桌面(.rdp)', value: 'rdp' },
  { label: 'URL链接(.url)', value: 'url' },
  { label: 'PDF文档(.pdf)', value: 'pdf' },
  { label: 'Word文档(.docx)', value: 'docx' },
  { label: '文本文件(.txt)', value: 'txt' },
  { label: '批处理(.bat)', value: 'bat' },
];

// 监听显示状态
watch(
  () => props.show,
  async (newVal) => {
    showModal.value = newVal;
    if (newVal) {
      // 每次打开对话框时都重新加载设置
      await loadSettings();
    }
  }
);

watch(showModal, (newVal) => {
  if (!newVal) {
    emit('update:show', false);
  }
});

// 加载设置
const loadSettings = async () => {
  loading.value = true;
  try {
    const result = await invoke<AppLauncherSettings>('get_launcher_settings');
    settings.value = result;
  } catch (error) {
    console.error('加载设置失败:', error);
    message.error('加载设置失败: ' + error);
  } finally {
    loading.value = false;
  }
};

// 检查是否是默认扩展名
const isDefaultExtension = (ext: string): boolean => {
  return defaultExtensions.includes(ext);
};

// 添加扩展名
const handleAddExtension = () => {
  const ext = newExtension.value.trim().toLowerCase().replace(/^\./, '');

  if (!ext) {
    message.warning('请输入文件后缀');
    return;
  }

  // 验证格式(只允许字母数字)
  if (!/^[a-z0-9]+$/i.test(ext)) {
    message.warning('文件后缀只能包含字母和数字');
    return;
  }

  if (settings.value.allowedExtensions.includes(ext)) {
    message.warning(`后缀 .${ext} 已存在`);
    return;
  }

  settings.value.allowedExtensions.push(ext);
  newExtension.value = '';
  message.success(`已添加 .${ext}`);
};

// 快速添加
const handleQuickAdd = (ext: string) => {
  settings.value.allowedExtensions.push(ext);
  message.success(`已添加 .${ext}`);
};

// 移除扩展名
const handleRemoveExtension = (ext: string) => {
  if (isDefaultExtension(ext)) {
    message.warning('默认文件类型不能删除');
    return;
  }

  settings.value.allowedExtensions = settings.value.allowedExtensions.filter((e) => e !== ext);
  message.success(`已移除 .${ext}`);
};

// 重置为默认
const handleReset = () => {
  settings.value.allowedExtensions = [...defaultExtensions];
  message.success('已重置为默认设置');
};

// 保存
const handleSave = async () => {
  loading.value = true;
  try {
    await invoke('update_launcher_settings', { settings: settings.value });
    message.success('设置已保存');
    emit('saved');
    showModal.value = false;
  } catch (error) {
    console.error('保存设置失败:', error);
    message.error('保存设置失败: ' + error);
  } finally {
    loading.value = false;
  }
};

// 取消
const handleCancel = () => {
  showModal.value = false;
};
</script>

<style scoped>
.settings-content {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.settings-section {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.section-header h3 {
  margin: 0 0 8px 0;
  font-size: 16px;
  font-weight: 600;
  color: #e2e8f0;
}

.section-desc {
  margin: 0;
  font-size: 13px;
  color: #94a3b8;
  line-height: 1.6;
}

.extensions-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  padding: 16px;
  background: rgba(30, 41, 59, 0.5);
  border-radius: 8px;
  min-height: 80px;
}

.extension-tag {
  font-size: 14px;
  font-family: 'Consolas', 'Monaco', monospace;
}

.add-extension {
  display: flex;
  gap: 12px;
  align-items: center;
}

.common-extensions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
  padding: 12px;
  background: rgba(30, 41, 59, 0.3);
  border-radius: 8px;
}

.common-extensions .label {
  font-size: 13px;
  color: #94a3b8;
  margin-right: 8px;
}

.quick-add-btn {
  font-size: 12px;
}

.help-alert {
  margin-top: 8px;
}

.help-alert :deep(.n-alert__header) {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
}

.help-alert ul {
  margin: 8px 0 0 0;
  padding-left: 20px;
  list-style: disc;
}

.help-alert li {
  margin: 4px 0;
  font-size: 13px;
  line-height: 1.6;
}

.footer-actions {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.right-actions {
  display: flex;
  gap: 12px;
}
</style>
