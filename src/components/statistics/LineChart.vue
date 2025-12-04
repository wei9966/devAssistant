<script setup lang="ts">
import { computed } from 'vue'
import { use } from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'
import { LineChart } from 'echarts/charts'
import {
  TitleComponent,
  TooltipComponent,
  GridComponent
} from 'echarts/components'
import VChart from 'vue-echarts'
import type { HourlyDistribution } from '@/api/statisticsApi'

use([
  CanvasRenderer,
  LineChart,
  TitleComponent,
  TooltipComponent,
  GridComponent
])

interface Props {
  data: HourlyDistribution[]
}

const props = defineProps<Props>()

const option = computed(() => {
  // 确保有完整的24小时数据
  const hourlyMap = new Map(props.data.map(item => [item.hour, item.count]))
  const hours = Array.from({ length: 24 }, (_, i) => i)
  const counts = hours.map(hour => hourlyMap.get(hour) || 0)

  return {
    title: {
      text: '时段分布',
      left: 'center',
      top: 10,
      textStyle: {
        color: '#fff',
        fontSize: 16
      }
    },
    tooltip: {
      trigger: 'axis',
      formatter: (params: any) => {
        const hour = params[0].axisValue
        const count = params[0].value
        return `${hour}:00 - ${hour}:59<br/>活动次数: ${count}`
      }
    },
    grid: {
      left: '3%',
      right: '4%',
      bottom: '3%',
      top: 60,
      containLabel: true
    },
    xAxis: {
      type: 'category',
      data: hours,
      boundaryGap: false,
      axisLabel: {
        color: '#999',
        formatter: '{value}:00'
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
        type: 'line',
        data: counts,
        smooth: true,
        symbol: 'circle',
        symbolSize: 6,
        lineStyle: {
          color: '#5470c6',
          width: 2
        },
        itemStyle: {
          color: '#5470c6'
        },
        areaStyle: {
          color: {
            type: 'linear',
            x: 0,
            y: 0,
            x2: 0,
            y2: 1,
            colorStops: [
              {
                offset: 0,
                color: 'rgba(84, 112, 198, 0.4)'
              },
              {
                offset: 1,
                color: 'rgba(84, 112, 198, 0.1)'
              }
            ]
          }
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
