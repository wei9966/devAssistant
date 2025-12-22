<template>
  <div class="date-navigation">
    <div class="nav-left">
      <n-button
        :type="isToday ? 'primary' : 'default'"
        size="small"
        @click="goToToday"
      >
        今天
      </n-button>
    </div>

    <div class="nav-center">
      <n-button
        quaternary
        circle
        size="small"
        @click="goPreviousDay"
      >
        <template #icon>
          <n-icon :component="ChevronBackOutline" />
        </template>
      </n-button>

      <n-date-picker
        v-model:value="dateValue"
        type="date"
        :is-date-disabled="disableFutureDate"
        @update:value="handleDateChange"
        class="date-picker"
      />

      <n-button
        quaternary
        circle
        size="small"
        :disabled="isToday"
        @click="goNextDay"
      >
        <template #icon>
          <n-icon :component="ChevronForwardOutline" />
        </template>
      </n-button>
    </div>

    <div class="nav-right">
      <span class="date-display">{{ formattedDate }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { NButton, NDatePicker, NIcon } from 'naive-ui'
import { ChevronBackOutline, ChevronForwardOutline } from '@vicons/ionicons5'
import dayjs from 'dayjs'

interface Props {
  date: Date
}

interface Emits {
  (e: 'update:date', value: Date): void
  (e: 'change', value: Date): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const dateValue = ref<number>(props.date.getTime())

// 监听 props 变化
watch(
  () => props.date,
  (newDate) => {
    dateValue.value = newDate.getTime()
  }
)

const isToday = computed(() => {
  return dayjs(props.date).isSame(dayjs(), 'day')
})

const formattedDate = computed(() => {
  const d = dayjs(props.date)
  if (isToday.value) {
    return '今天'
  }
  return d.format('YYYY年M月D日 dddd')
})

function disableFutureDate(ts: number): boolean {
  return dayjs(ts).isAfter(dayjs(), 'day')
}

function handleDateChange(value: number | null) {
  if (value) {
    const newDate = new Date(value)
    emit('update:date', newDate)
    emit('change', newDate)
  }
}

function goToToday() {
  const today = new Date()
  today.setHours(0, 0, 0, 0)
  dateValue.value = today.getTime()
  emit('update:date', today)
  emit('change', today)
}

function goPreviousDay() {
  const prev = dayjs(props.date).subtract(1, 'day').toDate()
  dateValue.value = prev.getTime()
  emit('update:date', prev)
  emit('change', prev)
}

function goNextDay() {
  if (!isToday.value) {
    const next = dayjs(props.date).add(1, 'day').toDate()
    dateValue.value = next.getTime()
    emit('update:date', next)
    emit('change', next)
  }
}
</script>

<style scoped>
.date-navigation {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  background: var(--card-bg);
  border: 1px solid var(--border-default);
  border-radius: 12px;
}

.nav-left {
  flex: 0 0 auto;
}

.nav-center {
  display: flex;
  align-items: center;
  gap: 8px;
}

.date-picker {
  width: 150px;
}

.nav-right {
  flex: 0 0 auto;
}

.date-display {
  font-size: 14px;
  color: var(--text-muted);
}

/* Naive UI 样式覆盖 */
:deep(.n-date-picker) {
  --n-border: 1px solid var(--border-default);
  --n-border-hover: 1px solid var(--accent-primary);
}

:deep(.n-button--quaternary-type) {
  --n-text-color: var(--text-muted);
  --n-text-color-hover: var(--accent-primary);
}
</style>
