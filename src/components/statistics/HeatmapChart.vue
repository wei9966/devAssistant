<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { use } from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'
import { HeatmapChart } from 'echarts/charts'
import {
  TitleComponent,
  TooltipComponent,
  VisualMapComponent,
  CalendarComponent
} from 'echarts/components'
import VChart from 'vue-echarts'
import type { HeatmapData } from '@/api/statisticsApi'

use([
  CanvasRenderer,
  HeatmapChart,
  TitleComponent,
  TooltipComponent,
  VisualMapComponent,
  CalendarComponent
])

interface Props {
  data: HeatmapData[]
}

const props = defineProps<Props>()

// 辅助函数：格式化本地日期，避免 toISOString() 的 UTC 时区问题
const formatLocalDate = (date: Date): string => {
  const year = date.getFullYear()
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  return `${year}-${month}-${day}`
}

const option = computed(() => {
  // 获取最近3个月的日期范围
  const endDate = new Date()
  const startDate = new Date()
  startDate.setMonth(startDate.getMonth() - 3)

  // 转换数据格式为 ECharts 需要的格式 [[date, count]]
  const chartData = props.data.map(item => [item.date, item.count])

  // 计算最大值用于视觉映射
  const maxCount = Math.max(...props.data.map(item => item.count), 1)

  return {
    tooltip: {
      position: 'top',
      formatter: (params: any) => {
        return `${params.value[0]}<br/>活动次数: ${params.value[1]}`
      }
    },
    visualMap: {
      min: 0,
      max: maxCount,
      type: 'piecewise',
      orient: 'horizontal',
      left: 'center',
      top: 10,
      pieces: [
        { min: 0, max: 0, label: '无活动', color: '#ebedf0' },
        { min: 1, max: Math.ceil(maxCount * 0.25), label: '少量', color: '#9be9a8' },
        { min: Math.ceil(maxCount * 0.25), max: Math.ceil(maxCount * 0.5), label: '中等', color: '#40c463' },
        { min: Math.ceil(maxCount * 0.5), max: Math.ceil(maxCount * 0.75), label: '较多', color: '#30a14e' },
        { min: Math.ceil(maxCount * 0.75), label: '大量', color: '#216e39' }
      ]
    },
    calendar: {
      top: 80,
      left: 40,
      right: 40,
      cellSize: ['auto', 15],
      range: [
        formatLocalDate(startDate),
        formatLocalDate(endDate)
      ],
      itemStyle: {
        borderWidth: 2,
        borderColor: '#1e1e1e'
      },
      yearLabel: { show: true },
      dayLabel: {
        firstDay: 1,
        nameMap: ['日', '一', '二', '三', '四', '五', '六']
      },
      monthLabel: {
        nameMap: 'cn'
      },
      splitLine: {
        show: true,
        lineStyle: {
          color: '#2a2a2a',
          width: 2
        }
      }
    },
    series: [
      {
        type: 'heatmap',
        coordinateSystem: 'calendar',
        data: chartData
      }
    ]
  }
})
</script>

<template>
  <v-chart :option="option" autoresize style="height: 250px" />
</template>

<style scoped>
</style>
