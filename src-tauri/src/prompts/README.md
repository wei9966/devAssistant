# Prompts Module - AI提示词模板

本模块包含DevAssistant项目中所有AI功能的提示词模板，用于Activity活动总结、智能提示生成、日报周报生成等功能。

## 模块结构

```
prompts/
├── mod.rs                    # 模块声明
├── activity_prompts.rs       # Activity活动总结相关提示词
├── tips_prompts.rs          # Tips智能提示相关提示词
└── report_prompts.rs        # 日报周报生成相关提示词
```

## 使用方法

### 1. Activity活动总结

```rust
use crate::prompts::activity_prompts::{
    ACTIVITY_SUMMARY_PROMPT,
    ACTIVITY_MERGE_PROMPT,
    ACTIVITY_CLASSIFICATION_PROMPT,
};

// 在VLM或AI服务中使用
let prompt = format!("{}\n\n输入数据：\n{}", ACTIVITY_SUMMARY_PROMPT, input_json);
let result = vlm_service.analyze_image(&screenshot_base64, &prompt).await?;
```

**提示词列表：**
- `ACTIVITY_SUMMARY_PROMPT`: 生成15分钟活动总结
- `ACTIVITY_MERGE_PROMPT`: 合并相似活动片段
- `ACTIVITY_CLASSIFICATION_PROMPT`: 自动分类活动类型

### 2. 智能提示生成

```rust
use crate::prompts::tips_prompts::{
    TIPS_PROMPT,
    FOCUS_ANALYSIS_PROMPT,
    WORK_RHYTHM_PROMPT,
    TIME_ALLOCATION_PROMPT,
};

// 生成个性化建议
let prompt = format!("{}\n\n工作数据：\n{}", TIPS_PROMPT, work_pattern_json);
let tip = ai_service.chat(&prompt).await?;
```

**提示词列表：**
- `TIPS_PROMPT`: 主提示词，生成个性化建议
- `FOCUS_ANALYSIS_PROMPT`: 专注度分析
- `WORK_RHYTHM_PROMPT`: 工作节奏建议
- `TIME_ALLOCATION_PROMPT`: 时间分配优化

### 3. 日报周报生成

```rust
use crate::prompts::report_prompts::{
    DAILY_REPORT_PROMPT,
    WEEKLY_REPORT_PROMPT,
    MONTHLY_REPORT_PROMPT,
    TIME_BLOCK_SUMMARY_PROMPT,
};

// 生成日报
let prompt = format!("{}\n\n活动数据：\n{}", DAILY_REPORT_PROMPT, daily_data_json);
let report = ai_service.chat(&prompt).await?;
```

**提示词列表：**
- `DAILY_REPORT_PROMPT`: 日报生成（按时段分块）
- `WEEKLY_REPORT_PROMPT`: 周报生成
- `MONTHLY_REPORT_PROMPT`: 月报生成
- `TIME_BLOCK_SUMMARY_PROMPT`: 时段快速摘要

## 输出格式说明

所有提示词都要求AI返回**严格的JSON格式**，便于程序解析和处理。

### Activity Summary 输出示例

```json
{
  "title": "开发用户认证模块",
  "description": "完成JWT认证逻辑实现，编写20+单元测试用例，代码覆盖率达到85%。",
  "keywords": ["认证", "JWT", "单元测试"],
  "activityBreakdown": {
    "coding": 70,
    "browsing": 20,
    "document": 10,
    "meeting": 0,
    "communication": 0,
    "other": 0
  },
  "importance": 4,
  "keyInsights": [
    "核心功能实现完成，代码质量良好",
    "测试覆盖率达标，为后续优化提供保障"
  ]
}
```

### Tips 输出示例

```json
{
  "tip": "您已经连续工作2小时了，建议休息5-10分钟，放松眼睛和身体。",
  "category": "health",
  "priority": "high",
  "actionable": true,
  "actions": [
    {
      "label": "开始5分钟休息",
      "action": "start_break",
      "duration": 5
    }
  ],
  "icon": "coffee",
  "dismissible": true
}
```

### Daily Report 输出示例

```json
{
  "summary": "今日主要完成用户认证模块开发，实现核心功能并完成单元测试。工作时长8.5小时。",
  "highlights": [
    {
      "time": "09:00-12:00",
      "title": "用户认证模块开发",
      "content": "完成JWT认证逻辑实现...",
      "achievements": [
        "实现JWT认证核心功能",
        "编写20+单元测试用例"
      ]
    }
  ],
  "insights": [
    {
      "type": "productivity",
      "content": "上午深度工作时段效率最高",
      "score": 0.85
    }
  ],
  "recommendations": [
    "建议明天重点关注性能测试"
  ],
  "statistics": {
    "coreMetrics": {
      "deepWorkHours": 5.5,
      "shallowWorkHours": 2.5,
      "focusScore": 0.78
    }
  },
  "tags": ["开发", "认证", "测试"]
}
```

## 设计原则

1. **简洁性**: 输出内容简洁明了，避免冗余
2. **结构化**: 统一使用JSON格式，便于解析
3. **可量化**: 提供具体的指标和数据
4. **可执行**: 建议要具体可行，避免空洞
5. **专业性**: 使用专业术语，保持客观

## 注意事项

1. 所有Prompt都使用原始字符串字面量（`r#"..."#`），避免转义问题
2. 输入数据格式在Prompt中有明确说明，使用时需要严格遵守
3. AI可能返回格式不标准的JSON，需要做好错误处理和解析
4. 建议在调用前后添加日志，便于调试和优化

## 后续扩展

可以根据实际需求添加更多提示词：
- 会议纪要生成
- 代码评审总结
- 学习进度分析
- 技能成长报告
- 团队协作分析

## 参考文档

- VLM Service: `src/services/vlm_service.rs`
- AI Service: `src/services/ai_service.rs`
- Activity Summary Service: `src/services/activity_summary_service.rs`
- Report Service: `src/services/report_service.rs`
