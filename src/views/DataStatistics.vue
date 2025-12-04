<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { NSpace, NCard, NDatePicker, NButton, NSpin, useMessage } from 'naive-ui'
import HeatmapChart from '@/components/statistics/HeatmapChart.vue'
import PieChart from '@/components/statistics/PieChart.vue'
import BarChart from '@/components/statistics/BarChart.vue'
import LineChart from '@/components/statistics/LineChart.vue'
import { statisticsApi } from '@/api/statisticsApi'
import type {
  HeatmapData,
  DailyTrend,
  HourlyDistribution,
  AppUsage,
  ActivityTypeStats
} from '@/api/statisticsApi'

const message = useMessage()

const loading = ref(false)
const dateRange = ref<[number, number] | null>(null)

// 数据
const heatmapData = ref<HeatmapData[]>([])
const dailyTrendData = ref<DailyTrend[]>([])
const hourlyDistributionData = ref<HourlyDistribution[]>([])
const appUsageData = ref<AppUsage[]>([])
const activityTypeData = ref<ActivityTypeStats[]>([])

// 转换应用使用数据为饼图格式
const appUsagePieData = computed(() => {
  return appUsageData.value.map(item => ({
    name: item.appName,
    value: item.count
  }))
})

// 转换活动类型数据为饼图格式
const activityTypePieData = computed(() => {
  return activityTypeData.value.map(item => ({
    name: item.activityType,
    value: item.count
  }))
})

// 加载所有数据
const loadData = async () => {
  loading.value = true
  try {
    let startDate: string | undefined
    let endDate: string | undefined

    if (dateRange.value) {
      // 使用本地时间格式化，避免 toISOString() 的 UTC 时区问题
      const formatLocalDate = (d: Date) => {
        const year = d.getFullYear()
        const month = String(d.getMonth() + 1).padStart(2, '0')
        const day = String(d.getDate()).padStart(2, '0')
        return `${year}-${month}-${day}`
      }
      startDate = formatLocalDate(new Date(dateRange.value[0]))
      endDate = formatLocalDate(new Date(dateRange.value[1]))
    }

    // 并行加载所有数据
    const [heatmap, dailyTrend, hourlyDist, appUsage, activityTypes] = await Promise.all([
      statisticsApi.getHeatmap(startDate, endDate),
      statisticsApi.getDailyTrend(30),
      statisticsApi.getHourlyDistribution(endDate),
      statisticsApi.getAppUsage(startDate, endDate),
      statisticsApi.getActivityTypes(startDate, endDate)
    ])

    heatmapData.value = heatmap
    dailyTrendData.value = dailyTrend
    hourlyDistributionData.value = hourlyDist
    appUsageData.value = appUsage
    activityTypeData.value = activityTypes

    message.success('数据加载成功')
  } catch (error) {
    console.error('加载数据失败:', error)
    message.error('加载数据失败: ' + (error as Error).message)
  } finally {
    loading.value = false
  }
}

// 重置日期范围
const resetDateRange = () => {
  dateRange.value = null
  loadData()
}

onMounted(() => {
  loadData()
})
</script>

<template>
  <div class="data-statistics">
    <NSpin :show="loading">
      <NSpace vertical :size="16">
        <!-- 日期选择器 -->
        <NCard title="日期筛选" size="small">
          <NSpace>
            <NDatePicker
              v-model:value="dateRange"
              type="daterange"
              clearable
              placeholder="选择日期范围"
              style="width: 300px"
            />
            <NButton type="primary" @click="loadData">
              查询
            </NButton>
            <NButton @click="resetDateRange">
              重置
            </NButton>
          </NSpace>
        </NCard>

        <!-- 热力图 -->
        <NCard title="活动热力图" size="small">
          <HeatmapChart :data="heatmapData" />
        </NCard>

        <!-- 饼图行 -->
        <div class="chart-row">
          <NCard title="" size="small">
            <PieChart :data="appUsagePieData" title="应用使用分布" />
          </NCard>
          <NCard title="" size="small">
            <PieChart :data="activityTypePieData" title="活动类型分布" />
          </NCard>
        </div>

        <!-- 柱状图和折线图行 -->
        <div class="chart-row">
          <NCard title="" size="small">
            <BarChart :data="dailyTrendData" />
          </NCard>
          <NCard title="" size="small">
            <LineChart :data="hourlyDistributionData" />
          </NCard>
        </div>
      </NSpace>
    </NSpin>
  </div>
</template>

<style scoped>
.data-statistics {
  padding: 16px;
  height: 100%;
  overflow-y: auto;
}

/* 让 NSpace 横向布局时占满宽度 */
:deep(.n-space) {
  width: 100%;
}

.chart-row {
  display: flex;
  gap: 16px;
  width: 100%;
}

.chart-row > * {
  flex: 1;
  min-width: 0;
}
</style>
