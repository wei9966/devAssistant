// Report Generation Prompts
// 日报和周报生成提示词模板

/// 日报生成主提示词
/// 基于全天活动数据生成结构化日报
pub const DAILY_REPORT_PROMPT: &str = r#"你是一个专业的工作日报撰写专家。请基于用户一天的活动记录，生成一份结构化、专业的工作日报。

**输入数据格式：**
```json
{
  "date": "2024-01-01",
  "workHours": {
    "start": "09:00:00",
    "end": "18:30:00",
    "totalMinutes": 510,
    "activeMinutes": 450,
    "breakMinutes": 60
  },
  "activityBlocks": [
    {
      "timeRange": "09:00-10:30",
      "title": "开发用户认证模块",
      "type": "coding",
      "description": "实现JWT认证逻辑",
      "importance": 4
    },
    {
      "timeRange": "10:30-11:00",
      "title": "代码评审",
      "type": "communication",
      "description": "审查团队成员的PR",
      "importance": 3
    }
  ],
  "statistics": {
    "appUsage": {
      "Visual Studio Code": 240,
      "Google Chrome": 120,
      "Slack": 45
    },
    "activityDistribution": {
      "coding": 60,
      "browsing": 20,
      "communication": 12,
      "document": 5,
      "other": 3
    }
  },
  "completedTasks": [
    "实现用户认证接口",
    "修复登录页面bug",
    "编写API文档"
  ]
}
```

**生成要求：**

1. **时间段分块总结**
   - 按2-3小时分组
   - 每个时段突出核心工作
   - 保持时间线连贯性

2. **内容结构**
   - 简洁明了，重点突出
   - 量化成果（如完成3个功能点）
   - 体现工作价值

3. **专业性**
   - 使用专业术语
   - 客观描述事实
   - 避免主观评价

**输出格式（严格JSON）：**
```json
{
  "summary": "今日主要完成用户认证模块开发，实现核心功能并完成单元测试。参与2次代码评审，协助团队成员解决技术问题。工作时长8.5小时，代码产出量达标。",
  "highlights": [
    {
      "time": "09:00-12:00",
      "title": "用户认证模块开发",
      "content": "完成JWT认证逻辑实现，包括token生成、验证和刷新机制。编写完整的单元测试，覆盖率达到85%。",
      "achievements": [
        "实现JWT认证核心功能",
        "编写20+单元测试用例",
        "完成API接口文档"
      ]
    },
    {
      "time": "14:00-17:30",
      "title": "代码优化与评审",
      "content": "优化数据库查询性能，响应时间降低40%。参与2次代码评审，提出5条优化建议。",
      "achievements": [
        "优化3个慢查询",
        "审查2个PR共计500行代码",
        "修复2个潜在bug"
      ]
    }
  ],
  "insights": [
    {
      "type": "productivity",
      "content": "上午深度工作时段效率最高，完成了核心功能开发",
      "score": 0.85
    },
    {
      "type": "focus",
      "content": "专注度较好，任务切换频率适中，保持了工作连贯性",
      "score": 0.78
    },
    {
      "type": "collaboration",
      "content": "参与团队协作时间占比12%，沟通效率良好",
      "score": 0.80
    }
  ],
  "recommendations": [
    "建议明天重点关注性能测试和压力测试",
    "下午时段专注度略有下降，可以调整工作节奏",
    "文档编写时间较少，建议增加技术文档产出"
  ],
  "statistics": {
    "coreMetrics": {
      "deepWorkHours": 5.5,
      "shallowWorkHours": 2.5,
      "breakHours": 1.0,
      "focusScore": 0.78,
      "productivityScore": 0.82
    },
    "timeDistribution": {
      "coding": 60,
      "browsing": 20,
      "communication": 12,
      "document": 5,
      "other": 3
    },
    "topApps": [
      {"name": "Visual Studio Code", "minutes": 240, "percentage": 53.3},
      {"name": "Google Chrome", "minutes": 120, "percentage": 26.7},
      {"name": "Slack", "minutes": 45, "percentage": 10.0}
    ]
  },
  "tags": ["开发", "认证", "代码评审", "性能优化"]
}
```

**字段说明：**
- summary: 一句话总结（50-100字，高度概括全天工作）
- highlights: 时段亮点（2-4个主要工作时段，每个包含标题、内容和成果）
- insights: 工作洞察（2-4条，分析工作模式和效率）
- recommendations: 改进建议（2-4条，具体可行）
- statistics: 统计数据（工作时长、分布、应用使用等）
- tags: 关键标签（3-8个，便于分类和搜索）

**注意事项：**
- 突出成果和价值，而非单纯罗列活动
- 使用量化指标增强说服力
- 洞察要有依据，避免空洞评价
- 建议要具体，可执行
- 保持专业客观的语气
- 只返回JSON格式，不要其他文字
"#;

/// 周报生成提示词
/// 基于一周的数据生成周报
pub const WEEKLY_REPORT_PROMPT: &str = r#"你是一个专业的周报撰写专家。请基于用户一周的活动数据，生成一份结构化、全面的工作周报。

**输入数据格式：**
```json
{
  "weekRange": "2024-01-01 ~ 2024-01-07",
  "dailyReports": [
    {
      "date": "2024-01-01",
      "summary": "完成用户认证模块",
      "highlights": ["..."],
      "tags": ["开发", "认证"]
    }
  ],
  "weekStatistics": {
    "totalWorkHours": 42.5,
    "deepWorkHours": 30.0,
    "productivityScore": 0.78,
    "focusScore": 0.75
  }
}
```

**输出格式：**
```json
{
  "weekSummary": "本周完成3个核心功能模块开发，修复12个bug，代码评审8次。工作总时长42.5小时，深度工作占比71%，整体产出质量良好。",
  "keyAchievements": [
    {
      "category": "开发",
      "items": [
        "完成用户认证模块（JWT + OAuth2）",
        "实现数据缓存策略，性能提升40%",
        "重构核心业务逻辑，代码可维护性提升"
      ]
    },
    {
      "category": "质量",
      "items": [
        "修复12个bug，其中3个为P0级别",
        "代码评审8次，提出优化建议15条",
        "单元测试覆盖率从60%提升至80%"
      ]
    }
  ],
  "weeklyInsights": [
    "周一至周三产出最高，专注度保持在0.8以上",
    "周四下午会议较多，影响深度工作时间",
    "周五下午效率略有下降，可优化时间安排"
  ],
  "trends": {
    "productivity": {
      "trend": "stable",
      "average": 0.78,
      "peak": "周三",
      "analysis": "整体产出稳定，保持较好的工作节奏"
    },
    "focus": {
      "trend": "improving",
      "average": 0.75,
      "analysis": "专注度逐步提升，任务切换频率降低"
    }
  },
  "nextWeekPlan": [
    "重点完成支付模块开发",
    "优化数据库查询性能",
    "补充技术文档和API文档"
  ]
}
```

只返回JSON格式。
"#;

/// 月报生成提示词
/// 基于月度数据生成月度总结
pub const MONTHLY_REPORT_PROMPT: &str = r#"你是月度工作总结专家。请基于用户一个月的工作数据，生成全面的月度工作报告。

**关注重点：**
1. 核心项目进展和里程碑
2. 技能成长和能力提升
3. 工作模式和效率趋势
4. 团队协作和贡献
5. 下月目标和规划

**输入格式：**
```json
{
  "month": "2024-01",
  "weeklyReports": [...],
  "monthStatistics": {
    "totalWorkDays": 22,
    "totalWorkHours": 176,
    "projectsCompleted": 3,
    "tasksCompleted": 45
  }
}
```

**输出格式：**
```json
{
  "monthSummary": "本月完成3个重点项目，交付5个核心功能模块。工作时长176小时，深度工作占比68%。技术能力持续提升，团队协作顺畅。",
  "milestones": [
    {
      "date": "2024-01-15",
      "title": "用户系统上线",
      "description": "完成用户认证、权限管理等核心功能，支持10万+用户"
    }
  ],
  "skillGrowth": [
    "深入学习Rust异步编程，掌握tokio框架",
    "实践DDD架构模式，提升系统设计能力",
    "学习性能优化技巧，解决实际性能瓶颈"
  ],
  "achievements": {
    "development": ["完成5个核心模块", "编写8000+行代码"],
    "quality": ["修复32个bug", "代码评审25次"],
    "collaboration": ["技术分享2次", "指导新人3名"]
  },
  "trends": {
    "efficiency": "整体效率稳步提升，月底较月初提高15%",
    "focus": "专注度保持在0.75以上，深度工作时间充足",
    "balance": "工作时长适中，保持了良好的工作生活平衡"
  },
  "nextMonthGoals": [
    "完成支付系统和订单系统开发",
    "优化系统性能，响应时间降低50%",
    "完善技术文档和团队知识库"
  ]
}
```

只返回JSON格式。
"#;

/// 时段总结提示词
/// 用于生成特定时段的快速摘要（用于日报分块）
pub const TIME_BLOCK_SUMMARY_PROMPT: &str = r#"你是时段总结助手。请为指定时段的活动生成简洁摘要。

**输入格式：**
```json
{
  "timeRange": "09:00-12:00",
  "activities": [
    {
      "time": "09:00-10:30",
      "title": "开发认证模块",
      "type": "coding",
      "importance": 4
    },
    {
      "time": "10:30-11:00",
      "title": "技术讨论",
      "type": "communication",
      "importance": 3
    }
  ]
}
```

**输出格式：**
```json
{
  "timeRange": "09:00-12:00",
  "title": "认证模块开发与技术讨论",
  "summary": "完成用户认证核心功能开发，实现JWT token机制。参与技术方案讨论，确定数据库架构方案。",
  "mainActivity": "coding",
  "productivity": 0.85,
  "achievements": [
    "实现JWT认证逻辑",
    "编写单元测试15个",
    "完成技术方案设计"
  ]
}
```

只返回JSON格式，摘要控制在2-3句话内。
"#;
