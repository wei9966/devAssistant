<template>
  <div
    ref="containerRef"
    class="virtual-grid-container custom-scrollbar"
    :class="{ 'list-mode': isListMode }"
    @scroll="handleScroll"
  >
    <!-- 置顶应用区域（不使用虚拟滚动，因为数量通常较少） -->
    <slot name="pinned"></slot>

    <!-- 虚拟滚动区域 -->
    <div v-if="items.length > 0" class="virtual-grid-wrapper">
      <slot name="header"></slot>

      <!-- 虚拟化容器 -->
      <div
        class="virtual-grid-inner"
        :style="innerStyle"
      >
        <!-- 只渲染可见的项目 -->
        <div
          v-for="item in visibleItems"
          :key="item.data.id"
          class="virtual-grid-item"
          :style="getItemStyle(item)"
        >
          <slot name="item" :item="item.data" :index="item.index"></slot>
        </div>
      </div>
    </div>

    <!-- 空状态 -->
    <slot v-if="items.length === 0" name="empty"></slot>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, nextTick, type CSSProperties } from 'vue';

interface Props {
  items: any[];
  itemWidth?: number;
  itemHeight?: number;
  gap?: number;
  buffer?: number; // 额外渲染的缓冲行数
}

const props = withDefaults(defineProps<Props>(), {
  itemWidth: 200,
  itemHeight: 160,
  gap: 16,
  buffer: 2,
});

const containerRef = ref<HTMLElement | null>(null);
const scrollTop = ref(0);
const containerWidth = ref(0);
const containerHeight = ref(0);

// 是否为列表模式（itemWidth 为 0 或非常小时）
const isListMode = computed(() => props.itemWidth <= 0);

// 实际使用的项目宽度
const actualItemWidth = computed(() => {
  if (isListMode.value) {
    return containerWidth.value - 16; // 留一点边距
  }
  return props.itemWidth;
});

// 计算每行可容纳的项目数
const columnsCount = computed(() => {
  if (isListMode.value) return 1;
  if (containerWidth.value === 0) return 1;
  // 计算实际可用宽度能放多少列（考虑gap）
  const availableWidth = containerWidth.value;
  const itemTotalWidth = actualItemWidth.value + props.gap;
  return Math.max(1, Math.floor((availableWidth + props.gap) / itemTotalWidth));
});

// 计算总行数
const rowsCount = computed(() => {
  return Math.ceil(props.items.length / columnsCount.value);
});

// 计算总高度
const totalHeight = computed(() => {
  const rowHeight = props.itemHeight + props.gap;
  return rowsCount.value * rowHeight - props.gap;
});

// 内部容器样式
const innerStyle = computed((): CSSProperties => {
  if (isListMode.value) {
    return {
      display: 'flex',
      flexDirection: 'column',
      gap: `${props.gap}px`,
    };
  }
  return {
    height: `${totalHeight.value}px`,
    position: 'relative',
  };
});

// 计算可见范围内的行
const visibleRange = computed(() => {
  if (isListMode.value) {
    // 列表模式下也使用虚拟滚动
    const rowHeight = props.itemHeight + props.gap;
    const startRow = Math.max(0, Math.floor(scrollTop.value / rowHeight) - props.buffer);
    const visibleRows = Math.ceil(containerHeight.value / rowHeight) + props.buffer * 2;
    const endRow = Math.min(props.items.length, startRow + visibleRows);
    return { startRow, endRow };
  }

  const rowHeight = props.itemHeight + props.gap;
  const startRow = Math.max(0, Math.floor(scrollTop.value / rowHeight) - props.buffer);
  const visibleRows = Math.ceil(containerHeight.value / rowHeight) + props.buffer * 2;
  const endRow = Math.min(rowsCount.value, startRow + visibleRows);
  return { startRow, endRow };
});

// 计算可见的项目（带位置信息）
const visibleItems = computed(() => {
  const result: Array<{ data: any; index: number; left: number; top: number }> = [];
  const { startRow, endRow } = visibleRange.value;

  if (isListMode.value) {
    // 列表模式：每行一个项目
    for (let i = startRow; i < endRow; i++) {
      if (i >= props.items.length) break;
      result.push({
        data: props.items[i],
        index: i,
        left: 0,
        top: i * (props.itemHeight + props.gap),
      });
    }
    return result;
  }

  // 网格模式
  const cols = columnsCount.value;
  const rowHeight = props.itemHeight + props.gap;
  const colWidth = actualItemWidth.value + props.gap;

  for (let row = startRow; row < endRow; row++) {
    for (let col = 0; col < cols; col++) {
      const index = row * cols + col;
      if (index >= props.items.length) break;

      result.push({
        data: props.items[index],
        index,
        left: col * colWidth,
        top: row * rowHeight,
      });
    }
  }

  return result;
});

// 获取项目样式
const getItemStyle = (item: { left: number; top: number }): CSSProperties => {
  if (isListMode.value) {
    return {
      width: '100%',
    };
  }
  return {
    position: 'absolute',
    left: `${item.left}px`,
    top: `${item.top}px`,
    width: `${actualItemWidth.value}px`,
    height: `${props.itemHeight}px`,
  };
};

// 处理滚动
const handleScroll = () => {
  if (containerRef.value) {
    scrollTop.value = containerRef.value.scrollTop;
  }
};

// 更新容器尺寸
const updateContainerSize = () => {
  if (containerRef.value) {
    containerWidth.value = containerRef.value.clientWidth;
    containerHeight.value = containerRef.value.clientHeight;
  }
};

// 使用 ResizeObserver 监听容器大小变化
let resizeObserver: ResizeObserver | null = null;

onMounted(() => {
  nextTick(() => {
    updateContainerSize();
  });

  if (containerRef.value) {
    resizeObserver = new ResizeObserver(() => {
      updateContainerSize();
    });
    resizeObserver.observe(containerRef.value);
  }
});

onUnmounted(() => {
  if (resizeObserver) {
    resizeObserver.disconnect();
  }
});

// 监听items变化，重新计算
watch(() => props.items.length, () => {
  nextTick(() => {
    updateContainerSize();
  });
});

// 暴露滚动方法
const scrollToTop = () => {
  if (containerRef.value) {
    containerRef.value.scrollTop = 0;
    scrollTop.value = 0;
  }
};

defineExpose({
  scrollToTop,
});
</script>

<style scoped>
.virtual-grid-container {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  display: flex;
  flex-direction: column;
  gap: 32px;
}

.virtual-grid-wrapper {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.virtual-grid-inner {
  width: 100%;
}

.virtual-grid-item {
  transition: opacity 0.15s ease;
}

/* 列表模式样式 */
.virtual-grid-container.list-mode .virtual-grid-inner {
  height: auto !important;
  position: relative !important;
}

.virtual-grid-container.list-mode .virtual-grid-item {
  position: relative !important;
  left: auto !important;
  top: auto !important;
}

/* Custom Scrollbar */
.custom-scrollbar::-webkit-scrollbar {
  width: 8px;
}

.custom-scrollbar::-webkit-scrollbar-track {
  background: var(--bg-hover);
  border-radius: 4px;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background: var(--border-default);
  border-radius: 4px;
  transition: background 0.2s;
}

.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: var(--border-hover);
}
</style>
