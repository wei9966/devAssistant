<script setup lang="ts">
import { computed } from 'vue'
import { use } from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'
import { BarChart } from 'echarts/charts'
import {
  TitleComponent,
  TooltipComponent,
  LegendComponent,
  GridComponent
} from 'echarts/components'
import VChart from 'vue-echarts'
import type { DailyTrend } from '@/api/statisticsApi'

use([
  CanvasRenderer,
  BarChart,
  TitleComponent,
  TooltipComponent,
  LegendComponent,
  GridComponent
])

interface Props {
  data: DailyTrend[]
}

const props = defineProps<Props>()

const option = computed(() => {
  const dates = props.data.map(item => item.date)
  const codingData = props.data.map(item => item.codingCount)
  const browsingData = props.data.map(item => item.browsingCount)
  const documentData = props.data.map(item => item.documentCount)
  const otherData = props.data.map(item => item.otherCount)

  return {
    title: {
      text: '每日活动趋势',
      left: 'center',
      top: 10,
      textStyle: {
        color: '#fff',
        fontSize: 16
      }
    },
    tooltip: {
      trigger: 'axis',
      axisPointer: {
        type: 'shadow'
      }
    },
    legend: {
      data: ['编码', '浏览', '文档', '其他'],
      top: 40,
      textStyle: {
        color: '#fff'
      }
    },
    grid: {
      left: '3%',
      right: '4%',
      bottom: '3%',
      top: 80,
      containLabel: true
    },
    xAxis: {
      type: 'category',
      data: dates,
      axisLabel: {
        color: '#999',
        rotate: 45
      },
      axisLine: {
        lineStyle: {
          color: '#333'
        }
      }
    },
    yAxis: {
      type: 'value',
      axisLabel: {
        color: '#999'
      },
      axisLine: {
        lineStyle: {
          color: '#333'
        }
      },
      splitLine: {
        lineStyle: {
          color: '#2a2a2a'
        }
      }
    },
    series: [
      {
        name: '编码',
        type: 'bar',
        stack: 'total',
        data: codingData,
        itemStyle: {
          color: '#5470c6'
        }
      },
      {
        name: '浏览',
        type: 'bar',
        stack: 'total',
        data: browsingData,
        itemStyle: {
          color: '#91cc75'
        }
      },
      {
        name: '文档',
        type: 'bar',
        stack: 'total',
        data: documentData,
        itemStyle: {
          color: '#fac858'
        }
      },
      {
        name: '其他',
        type: 'bar',
        stack: 'total',
        data: otherData,
        itemStyle: {
          color: '#ee6666'
        }
      }
    ]
  }
})
</script>

<template>
  <v-chart :option="option" autoresize style="height: 350px" />
</template>

<style scoped>
</style>
