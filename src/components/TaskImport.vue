<template>
  <div class="task-import">
    <div class="import-header">
      <h3>批量导入任务</h3>
      <button @click="emit('close')" class="close-btn">×</button>
    </div>

    <div class="import-body">
      <!-- 导入说明 -->
      <div class="import-info">
        <p>支持 JSON 和 Excel 格式的任务数据导入。可以导入历史任务数据，包括创建时间、完成时间等信息。</p>
        <div class="actions">
          <button @click="downloadTemplate('json')" class="btn-secondary">
            📥 下载 JSON 模板
          </button>
          <button @click="downloadTemplate('excel')" class="btn-secondary">
            📥 下载 Excel 模板
          </button>
          <button @click="showHelp = !showHelp" class="btn-secondary">
            {{ showHelp ? '隐藏' : '显示' }}字段说明
          </button>
        </div>
      </div>

      <!-- 字段说明 -->
      <div v-if="showHelp" class="help-section">
        <h4>字段说明：</h4>
        <div class="help-content">
          <div class="help-column">
            <h5>JSON 格式：</h5>
            <ul>
              <li><strong>title</strong> (必填): 任务标题</li>
              <li><strong>description</strong>: 任务描述</li>
              <li><strong>category</strong>: 分类 (dev/ops/study/other)</li>
              <li><strong>priority</strong>: 优先级 (1=高, 2=中, 3=低)</li>
              <li><strong>status</strong>: 状态 (todo/active/done/deferred)</li>
              <li><strong>createdAt</strong>: 创建时间 (格式: 2024-01-15 10:00:00)</li>
              <li><strong>startedAt</strong>: 开始时间</li>
              <li><strong>completedAt</strong>: 完成时间</li>
              <li><strong>estimatedHours</strong>: 预估工时</li>
              <li><strong>actualHours</strong>: 实际工时</li>
              <li><strong>gitBranch</strong>: Git 分支</li>
              <li><strong>notes</strong>: 备注</li>
            </ul>
          </div>
          <div class="help-column">
            <h5>Excel 格式：</h5>
            <ul>
              <li><strong>任务标题</strong> (必填): 任务标题</li>
              <li><strong>任务描述</strong>: 任务描述</li>
              <li><strong>分类</strong>: dev/ops/study/other</li>
              <li><strong>优先级</strong>: 1=高, 2=中, 3=低</li>
              <li><strong>状态</strong>: todo/active/done/deferred</li>
              <li><strong>创建时间</strong>: 格式 2024-01-15 10:00:00</li>
              <li><strong>开始时间</strong>: 格式同上</li>
              <li><strong>完成时间</strong>: 格式同上</li>
              <li><strong>预估工时</strong>: 数字（小时）</li>
              <li><strong>实际工时</strong>: 数字（小时）</li>
              <li><strong>Git分支</strong>: 分支名称</li>
              <li><strong>备注</strong>: 备注信息</li>
            </ul>
          </div>
        </div>
      </div>

      <!-- 文件上传区域 -->
      <div class="upload-area">
        <input
          type="file"
          ref="fileInput"
          @change="handleFileSelect"
          accept=".json,.xlsx,.xls"
          style="display: none"
        />
        <div
          class="drop-zone"
          :class="{ 'drag-over': isDragOver }"
          @click="triggerFileInput"
          @dragover.prevent="isDragOver = true"
          @dragleave.prevent="isDragOver = false"
          @drop.prevent="handleFileDrop"
        >
          <div v-if="!selectedFile" class="drop-zone-content">
            <div class="upload-icon">📁</div>
            <p>点击选择文件或拖拽文件到此处</p>
            <p class="file-hint">支持 .json、.xlsx、.xls 格式</p>
          </div>
          <div v-else class="file-selected">
            <div class="file-icon">📄</div>
            <p class="file-name">{{ selectedFile.name }}</p>
            <p class="file-size">{{ formatFileSize(selectedFile.size) }}</p>
            <button @click.stop="clearFile" class="clear-file-btn">✕</button>
          </div>
        </div>
      </div>

      <!-- 预览数据 -->
      <div v-if="previewData.length > 0" class="preview-section">
        <h4>预览数据 (共 {{ previewData.length }} 条)</h4>
        <div class="preview-list">
          <div v-for="(task, index) in previewData.slice(0, 5)" :key="index" class="preview-item">
            <div class="preview-title">{{ task.title }}</div>
            <div class="preview-meta">
              <span class="badge" :class="`badge-${task.category || 'other'}`">
                {{ getCategoryLabel(task.category) }}
              </span>
              <span class="badge" :class="`badge-priority-${task.priority || 2}`">
                优先级: {{ task.priority || 2 }}
              </span>
              <span v-if="task.createdAt" class="time">{{ task.createdAt }}</span>
            </div>
          </div>
          <div v-if="previewData.length > 5" class="preview-more">
            还有 {{ previewData.length - 5 }} 条任务...
          </div>
        </div>
      </div>

      <!-- 导入结果 -->
      <div v-if="importResult" class="result-section">
        <div class="result-summary" :class="{ 'has-errors': importResult.failed > 0 }">
          <div class="result-item success">
            <span class="result-icon">✓</span>
            <span>成功: {{ importResult.success }}</span>
          </div>
          <div v-if="importResult.failed > 0" class="result-item error">
            <span class="result-icon">✕</span>
            <span>失败: {{ importResult.failed }}</span>
          </div>
        </div>
        <div v-if="importResult.errors.length > 0" class="error-list">
          <h4>错误详情：</h4>
          <ul>
            <li v-for="(error, index) in importResult.errors" :key="index">{{ error }}</li>
          </ul>
        </div>
      </div>

      <!-- 操作按钮 -->
      <div class="import-actions">
        <button
          @click="handleImport"
          :disabled="!selectedFile || importing"
          class="btn-primary"
        >
          {{ importing ? '导入中...' : '开始导入' }}
        </button>
        <button @click="emit('close')" class="btn-secondary">
          取消
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { taskApi, type ImportTask, type ImportResult } from '@/api/taskApi';
import { save } from '@tauri-apps/plugin-dialog';
import { writeTextFile, writeFile } from '@tauri-apps/plugin-fs';
import * as XLSX from 'xlsx';

const emit = defineEmits<{
  close: [];
  success: [];
}>();

const fileInput = ref<HTMLInputElement>();
const selectedFile = ref<File | null>(null);
const previewData = ref<ImportTask[]>([]);
const importResult = ref<ImportResult | null>(null);
const importing = ref(false);
const isDragOver = ref(false);
const showHelp = ref(false);

// 触发文件选择
const triggerFileInput = () => {
  fileInput.value?.click();
};

// 处理文件选择
const handleFileSelect = (event: Event) => {
  const target = event.target as HTMLInputElement;
  if (target.files && target.files.length > 0) {
    handleFile(target.files[0]);
  }
};

// 处理文件拖放
const handleFileDrop = (event: DragEvent) => {
  isDragOver.value = false;
  if (event.dataTransfer?.files && event.dataTransfer.files.length > 0) {
    handleFile(event.dataTransfer.files[0]);
  }
};

// 处理文件
const handleFile = async (file: File) => {
  const fileName = file.name.toLowerCase();
  const isJson = fileName.endsWith('.json');
  const isExcel = fileName.endsWith('.xlsx') || fileName.endsWith('.xls');

  if (!isJson && !isExcel) {
    alert('只支持 JSON、XLSX、XLS 格式文件');
    return;
  }

  selectedFile.value = file;
  importResult.value = null;

  try {
    if (isJson) {
      // 解析 JSON
      const text = await file.text();
      const data = JSON.parse(text);
      previewData.value = Array.isArray(data) ? data : [data];
    } else {
      // 解析 Excel
      const arrayBuffer = await file.arrayBuffer();
      const workbook = XLSX.read(arrayBuffer, { type: 'array' });
      const firstSheet = workbook.Sheets[workbook.SheetNames[0]];
      const jsonData = XLSX.utils.sheet_to_json(firstSheet) as any[];

      // 转换 Excel 数据到 ImportTask 格式
      previewData.value = jsonData.map(row => ({
        title: row['任务标题'] || row['title'] || '',
        description: row['任务描述'] || row['description'] || undefined,
        category: row['分类'] || row['category'] || undefined,
        priority: row['优先级'] || row['priority'] || undefined,
        status: row['状态'] || row['status'] || undefined,
        gitBranch: row['Git分支'] || row['gitBranch'] || undefined,
        createdAt: row['创建时间'] || row['createdAt'] || undefined,
        startedAt: row['开始时间'] || row['startedAt'] || undefined,
        completedAt: row['完成时间'] || row['completedAt'] || undefined,
        estimatedHours: row['预估工时'] || row['estimatedHours'] || undefined,
        actualHours: row['实际工时'] || row['actualHours'] || undefined,
        notes: row['备注'] || row['notes'] || undefined,
      }));
    }
  } catch (error) {
    alert('文件解析失败: ' + (error as Error).message);
    clearFile();
  }
};

// 清除文件
const clearFile = () => {
  selectedFile.value = null;
  previewData.value = [];
  importResult.value = null;
  if (fileInput.value) {
    fileInput.value.value = '';
  }
};

// 执行导入
const handleImport = async () => {
  if (!selectedFile.value || previewData.value.length === 0) return;

  importing.value = true;
  try {
    const result = await taskApi.importTasks(previewData.value);
    importResult.value = result;

    if (result.failed === 0) {
      setTimeout(() => {
        emit('success');
        emit('close');
      }, 1500);
    }
  } catch (error) {
    alert('导入失败: ' + (error as Error).message);
  } finally {
    importing.value = false;
  }
};

// 下载模板
const downloadTemplate = async (type: 'json' | 'excel') => {
  try {
    if (type === 'json') {
      // 下载 JSON 模板
      const template = await taskApi.getImportTemplate();

      const filePath = await save({
        defaultPath: 'task-import-template.json',
        filters: [{
          name: 'JSON',
          extensions: ['json']
        }]
      });

      if (filePath) {
        await writeTextFile(filePath, template);
        alert('JSON 模板已保存到: ' + filePath);
      }
    } else {
      // 下载 Excel 模板
      const excelData = [
        {
          '任务标题': '示例任务1',
          '任务描述': '这是一个示例任务的描述',
          '分类': 'dev',
          '优先级': 2,
          '状态': 'todo',
          'Git分支': '',
          '创建时间': '2024-01-15 10:00:00',
          '开始时间': '',
          '完成时间': '',
          '预估工时': 4.0,
          '实际工时': '',
          '备注': '备注信息'
        },
        {
          '任务标题': '示例任务2（已完成）',
          '任务描述': '这是一个已完成的任务',
          '分类': 'ops',
          '优先级': 1,
          '状态': 'done',
          'Git分支': 'feature/example',
          '创建时间': '2024-01-10 09:00:00',
          '开始时间': '2024-01-10 09:30:00',
          '完成时间': '2024-01-11 18:00:00',
          '预估工时': 8.0,
          '实际工时': 9.5,
          '备注': ''
        }
      ];

      // 创建工作簿
      const worksheet = XLSX.utils.json_to_sheet(excelData);
      const workbook = XLSX.utils.book_new();
      XLSX.utils.book_append_sheet(workbook, worksheet, '任务列表');

      // 设置列宽
      worksheet['!cols'] = [
        { wch: 20 }, // 任务标题
        { wch: 30 }, // 任务描述
        { wch: 10 }, // 分类
        { wch: 10 }, // 优先级
        { wch: 10 }, // 状态
        { wch: 20 }, // Git分支
        { wch: 20 }, // 创建时间
        { wch: 20 }, // 开始时间
        { wch: 20 }, // 完成时间
        { wch: 12 }, // 预估工时
        { wch: 12 }, // 实际工时
        { wch: 30 }, // 备注
      ];

      // 生成 Excel 文件
      const excelBuffer = XLSX.write(workbook, { type: 'array', bookType: 'xlsx' });

      const filePath = await save({
        defaultPath: 'task-import-template.xlsx',
        filters: [{
          name: 'Excel',
          extensions: ['xlsx']
        }]
      });

      if (filePath) {
        await writeFile(filePath, new Uint8Array(excelBuffer));
        alert('Excel 模板已保存到: ' + filePath);
      }
    }
  } catch (error) {
    console.error('下载模板失败:', error);
    alert('下载模板失败: ' + (error as Error).message);
  }
};

// 格式化文件大小
const formatFileSize = (bytes: number): string => {
  if (bytes < 1024) return bytes + ' B';
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(2) + ' KB';
  return (bytes / (1024 * 1024)).toFixed(2) + ' MB';
};

// 获取分类标签
const getCategoryLabel = (category?: string): string => {
  const labels: Record<string, string> = {
    dev: '开发',
    ops: '运维',
    study: '学习',
    other: '其他',
  };
  return labels[category || 'other'] || '其他';
};
</script>

<style scoped>
.task-import {
  background: white;
  border-radius: 12px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
  max-width: 700px;
  max-height: 90vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.import-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px;
  border-bottom: 1px solid #e5e7eb;
}

.import-header h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: #1f2937;
}

.close-btn {
  background: none;
  border: none;
  font-size: 28px;
  cursor: pointer;
  color: #9ca3af;
  line-height: 1;
  padding: 0;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  transition: all 0.2s;
}

.close-btn:hover {
  background: #f3f4f6;
  color: #4b5563;
}

.import-body {
  padding: 24px;
  overflow-y: auto;
  flex: 1;
}

.import-info {
  background: #f0f9ff;
  border: 1px solid #bae6fd;
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 20px;
}

.import-info p {
  margin: 0 0 12px 0;
  color: #0c4a6e;
  font-size: 14px;
}

.actions {
  display: flex;
  gap: 8px;
}

.help-section {
  background: #fef3c7;
  border: 1px solid #fde68a;
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 20px;
}

.help-section h4 {
  margin: 0 0 12px 0;
  color: #92400e;
  font-size: 14px;
  font-weight: 600;
}

.help-content {
  display: flex;
  gap: 20px;
}

.help-column {
  flex: 1;
}

.help-column h5 {
  margin: 0 0 8px 0;
  color: #92400e;
  font-size: 13px;
  font-weight: 600;
}

.help-section ul {
  margin: 0;
  padding-left: 20px;
  color: #92400e;
  font-size: 12px;
}

.help-section li {
  margin-bottom: 4px;
}

.upload-area {
  margin-bottom: 20px;
}

.drop-zone {
  border: 2px dashed #d1d5db;
  border-radius: 12px;
  padding: 40px 20px;
  text-align: center;
  cursor: pointer;
  transition: all 0.3s;
  background: #f9fafb;
}

.drop-zone:hover,
.drop-zone.drag-over {
  border-color: #3b82f6;
  background: #eff6ff;
}

.drop-zone-content .upload-icon {
  font-size: 48px;
  margin-bottom: 12px;
}

.drop-zone-content p {
  margin: 4px 0;
  color: #6b7280;
}

.file-hint {
  font-size: 12px;
  color: #9ca3af;
}

.file-selected {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.file-icon {
  font-size: 40px;
}

.file-name {
  font-weight: 500;
  color: #1f2937;
  margin: 0;
}

.file-size {
  font-size: 13px;
  color: #6b7280;
  margin: 0;
}

.clear-file-btn {
  position: absolute;
  top: -10px;
  right: -10px;
  background: #ef4444;
  color: white;
  border: none;
  border-radius: 50%;
  width: 24px;
  height: 24px;
  cursor: pointer;
  font-size: 14px;
  line-height: 1;
  transition: all 0.2s;
}

.clear-file-btn:hover {
  background: #dc2626;
  transform: scale(1.1);
}

.preview-section {
  margin-bottom: 20px;
}

.preview-section h4 {
  margin: 0 0 12px 0;
  font-size: 14px;
  font-weight: 600;
  color: #1f2937;
}

.preview-list {
  background: #f9fafb;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  padding: 12px;
  max-height: 200px;
  overflow-y: auto;
}

.preview-item {
  padding: 8px;
  background: white;
  border-radius: 6px;
  margin-bottom: 8px;
}

.preview-item:last-child {
  margin-bottom: 0;
}

.preview-title {
  font-size: 14px;
  font-weight: 500;
  color: #1f2937;
  margin-bottom: 6px;
}

.preview-meta {
  display: flex;
  gap: 8px;
  align-items: center;
  font-size: 12px;
}

.badge {
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 500;
}

.badge-dev { background: #dbeafe; color: #1e40af; }
.badge-ops { background: #fce7f3; color: #9f1239; }
.badge-study { background: #fef3c7; color: #92400e; }
.badge-other { background: #f3f4f6; color: #4b5563; }

.badge-priority-1 { background: #fee2e2; color: #991b1b; }
.badge-priority-2 { background: #fef3c7; color: #92400e; }
.badge-priority-3 { background: #e0e7ff; color: #3730a3; }

.time {
  color: #6b7280;
}

.preview-more {
  text-align: center;
  padding: 8px;
  color: #6b7280;
  font-size: 13px;
}

.result-section {
  margin-bottom: 20px;
}

.result-summary {
  display: flex;
  gap: 16px;
  padding: 16px;
  background: #f0fdf4;
  border: 1px solid #86efac;
  border-radius: 8px;
  margin-bottom: 12px;
}

.result-summary.has-errors {
  background: #fef2f2;
  border-color: #fecaca;
}

.result-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 500;
}

.result-item.success {
  color: #166534;
}

.result-item.error {
  color: #991b1b;
}

.result-icon {
  font-size: 18px;
}

.error-list {
  background: #fef2f2;
  border: 1px solid #fecaca;
  border-radius: 8px;
  padding: 12px;
}

.error-list h4 {
  margin: 0 0 8px 0;
  font-size: 13px;
  font-weight: 600;
  color: #991b1b;
}

.error-list ul {
  margin: 0;
  padding-left: 20px;
  color: #b91c1c;
  font-size: 12px;
}

.error-list li {
  margin-bottom: 4px;
}

.import-actions {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
  padding-top: 16px;
  border-top: 1px solid #e5e7eb;
}

.btn-primary,
.btn-secondary {
  padding: 10px 20px;
  border-radius: 8px;
  border: none;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-primary {
  background: #3b82f6;
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background: #2563eb;
}

.btn-primary:disabled {
  background: #9ca3af;
  cursor: not-allowed;
}

.btn-secondary {
  background: #f3f4f6;
  color: #4b5563;
}

.btn-secondary:hover {
  background: #e5e7eb;
}
</style>
