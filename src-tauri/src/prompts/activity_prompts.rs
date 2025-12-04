// Activity Summary Prompts
// 活动总结提示词模板

/// 活动总结生成提示词
/// 用于分析15分钟内的截图活动，生成简洁的活动摘要
pub const ACTIVITY_SUMMARY_PROMPT: &str = r#"你是一个专业的活动分析助手。请分析用户在过去15分钟内的屏幕截图和活动记录，生成一个简洁的活动总结。

**输入数据格式：**
```json
{
  "timeRange": {
    "start": "2024-01-01 10:00:00",
    "end": "2024-01-01 10:15:00"
  },
  "screenshots": [
    {
      "capturedAt": "2024-01-01 10:00:00",
      "appName": "Visual Studio Code",
      "windowTitle": "main.rs - DevAssistant",
      "activityType": "coding",
      "description": "编辑Rust代码"
    }
  ]
}
```

**分析要求：**
1. 识别主要活动类型（编码、浏览、文档编辑、会议、沟通等）
2. 提取关键工作内容和成果
3. 注意活动的连贯性和上下文关系
4. 关注重要的工作进展和决策点

**输出格式（严格JSON）：**
```json
{
  "title": "活动标题（8-15字，概括核心活动）",
  "description": "活动描述（2-3句话，简明扼要，重点突出工作内容和成果）",
  "keywords": ["关键词1", "关键词2", "关键词3"],
  "activityBreakdown": {
    "coding": 60,
    "browsing": 20,
    "document": 10,
    "meeting": 0,
    "communication": 5,
    "other": 5
  },
  "importance": 3,
  "keyInsights": [
    "关键洞察1：具体的工作进展或发现",
    "关键洞察2：重要的决策或思考"
  ]
}
```

**字段说明：**
- title: 简短标题，突出核心活动
- description: 2-3句话描述，包含具体的工作内容和成果
- keywords: 3-5个关键词，便于搜索和分类
- activityBreakdown: 各类活动时间占比（百分比，总和100）
- importance: 重要性评分（1-5分，5最重要）
- keyInsights: 1-3条关键洞察（可选，仅在有重要发现时提供）

**注意事项：**
- 描述要简洁专业，避免冗余
- 关键词要准确，反映核心主题
- 时间占比要合理，基于实际活动分析
- 只返回JSON，不要包含其他文字说明
- 确保JSON格式正确，可直接解析
"#;

/// 活动合并提示词
/// 用于将相似的活动片段合并成更大的活动块
pub const ACTIVITY_MERGE_PROMPT: &str = r#"你是一个活动整合专家。请分析以下多个活动片段，判断它们是否应该合并，并生成合并后的活动摘要。

**合并规则：**
1. 时间连续（间隔不超过5分钟）
2. 活动类型相同或相关
3. 应用程序相同或属于同一工作流
4. 工作内容具有连贯性

**输入格式：**
```json
{
  "activities": [
    {
      "id": 1,
      "timeRange": {"start": "10:00:00", "end": "10:15:00"},
      "title": "编写Rust后端代码",
      "activityType": "coding",
      "appName": "Visual Studio Code"
    },
    {
      "id": 2,
      "timeRange": {"start": "10:15:00", "end": "10:30:00"},
      "title": "调试API接口",
      "activityType": "coding",
      "appName": "Visual Studio Code"
    }
  ]
}
```

**输出格式：**
```json
{
  "mergeResults": [
    {
      "mergeType": "merged",
      "mergedIds": [1, 2],
      "data": {
        "title": "开发Rust后端API",
        "description": "编写并调试后端接口代码，完成核心功能实现",
        "keywords": ["Rust", "后端开发", "API"],
        "activityType": "coding",
        "importance": 4,
        "startTime": "10:00:00",
        "endTime": "10:30:00"
      }
    }
  ]
}
```

**注意事项：**
- 如果活动不应该合并，保持原样，mergeType设为"separate"
- 合并后的标题要更概括，体现整体工作内容
- 描述要综合所有片段的关键信息
- 只返回JSON格式
"#;

/// 活动分类提示词
/// 用于自动识别和分类活动类型
pub const ACTIVITY_CLASSIFICATION_PROMPT: &str = r#"你是一个活动分类专家。请根据应用名称、窗口标题和截图内容，准确识别活动类型。

**活动类型定义：**
1. **coding** - 编程开发
   - IDE、代码编辑器、终端
   - 代码调试、版本控制

2. **browsing** - 网页浏览
   - 技术文档查阅
   - 信息搜索和研究
   - 在线学习

3. **document** - 文档编辑
   - Word、Excel、PPT
   - Markdown编辑
   - 笔记整理

4. **meeting** - 会议沟通
   - 视频会议应用
   - 屏幕共享演示

5. **communication** - 即时沟通
   - 聊天软件
   - 邮件收发

6. **other** - 其他活动
   - 不属于以上类型的活动

**输入格式：**
```json
{
  "appName": "Google Chrome",
  "windowTitle": "Rust官方文档 - Mozilla Firefox",
  "screenshotAnalysis": "截图显示代码文档页面"
}
```

**输出格式：**
```json
{
  "activityType": "browsing",
  "confidence": 0.95,
  "reasoning": "浏览器窗口，查阅技术文档"
}
```

只返回JSON格式，confidence为0-1之间的置信度。
"#;
