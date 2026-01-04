// Conversation Context Service
// 对话上下文追踪服务 - 实现对话连贯性增强

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;

/// 实体类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    /// 任务
    Task,
    /// 番茄钟
    Pomodoro,
    /// 项目
    Project,
    /// 标签
    Tag,
    /// SQL记录
    SqlRecord,
    /// 报告
    Report,
}

/// 对话主题
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConversationTopic {
    /// 任务管理
    TaskManagement,
    /// 番茄钟专注
    PomodoroFocus,
    /// SQL搜索
    SqlSearch,
    /// 报告生成
    ReportGeneration,
    /// 通用聊天
    GeneralChat,
}

/// 实体引用 - 记录对话中提及的实体
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityReference {
    /// 实体类型
    pub entity_type: EntityType,
    /// 实体ID（可能为空，如通过名称引用）
    pub id: Option<i64>,
    /// 实体名称
    pub name: String,
    /// 提及时的对话轮次
    pub mentioned_at_turn: usize,
    /// 附加元数据（如任务的优先级、标签等）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

impl EntityReference {
    /// 创建新的实体引用
    pub fn new(
        entity_type: EntityType,
        id: Option<i64>,
        name: String,
        mentioned_at_turn: usize,
    ) -> Self {
        Self {
            entity_type,
            id,
            name,
            mentioned_at_turn,
            metadata: None,
        }
    }

    /// 创建带元数据的实体引用
    pub fn with_metadata(
        entity_type: EntityType,
        id: Option<i64>,
        name: String,
        mentioned_at_turn: usize,
        metadata: HashMap<String, String>,
    ) -> Self {
        Self {
            entity_type,
            id,
            name,
            mentioned_at_turn,
            metadata: Some(metadata),
        }
    }

    /// 检查实体是否过期
    pub fn is_stale(&self, current_turn: usize, max_turns: usize) -> bool {
        current_turn.saturating_sub(self.mentioned_at_turn) > max_turns
    }
}

/// 对话上下文 - 追踪对话状态
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationContext {
    /// 最近提及的实体（按类型分类）
    pub recent_entities: HashMap<EntityType, Vec<EntityReference>>,
    /// 当前对话主题
    pub current_topic: Option<ConversationTopic>,
    /// 最后一次操作的实体
    pub last_operated_entity: Option<EntityReference>,
    /// 对话轮次计数
    pub turn_count: usize,
    /// 最大保留轮次（超过此轮次的实体将被清理）
    #[serde(default = "default_max_turns")]
    pub max_turns: usize,
}

/// 默认最大保留轮次
fn default_max_turns() -> usize {
    10
}

impl Default for ConversationContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ConversationContext {
    /// 创建新的对话上下文
    pub fn new() -> Self {
        Self {
            recent_entities: HashMap::new(),
            current_topic: None,
            last_operated_entity: None,
            turn_count: 0,
            max_turns: default_max_turns(),
        }
    }

    /// 创建带自定义最大轮次的对话上下文
    pub fn with_max_turns(max_turns: usize) -> Self {
        Self {
            recent_entities: HashMap::new(),
            current_topic: None,
            last_operated_entity: None,
            turn_count: 0,
            max_turns,
        }
    }

    /// 添加提及的实体
    pub fn add_entity(&mut self, entity: EntityReference) {
        let entity_type = entity.entity_type;
        let entities = self.recent_entities.entry(entity_type).or_insert_with(Vec::new);

        // 检查是否已存在相同ID的实体，如果存在则更新轮次
        if let Some(id) = entity.id {
            if let Some(existing) = entities.iter_mut().find(|e| e.id == Some(id)) {
                existing.mentioned_at_turn = self.turn_count;
                existing.name = entity.name;
                existing.metadata = entity.metadata;
                return;
            }
        }

        // 添加新实体
        entities.push(entity);

        // 限制列表长度，保留最近的N个
        const MAX_ENTITIES_PER_TYPE: usize = 20;
        if entities.len() > MAX_ENTITIES_PER_TYPE {
            entities.drain(0..entities.len() - MAX_ENTITIES_PER_TYPE);
        }
    }

    /// 批量添加实体
    pub fn add_entities(&mut self, entities: Vec<EntityReference>) {
        for entity in entities {
            self.add_entity(entity);
        }
    }

    /// 设置最后操作的实体
    pub fn set_last_operated(&mut self, entity: EntityReference) {
        // 同时添加到recent_entities中
        self.add_entity(entity.clone());
        self.last_operated_entity = Some(entity);
    }

    /// 更新对话主题
    pub fn update_topic(&mut self, topic: ConversationTopic) {
        self.current_topic = Some(topic);
    }

    /// 进入下一轮对话
    pub fn next_turn(&mut self) {
        self.turn_count += 1;
        // 自动清理过期实体
        self.clear_stale_entities();
    }

    /// 清理过期实体
    pub fn clear_stale_entities(&mut self) {
        let current_turn = self.turn_count;
        let max_turns = self.max_turns;

        for entities in self.recent_entities.values_mut() {
            entities.retain(|entity| !entity.is_stale(current_turn, max_turns));
        }

        // 清理空的实体列表
        self.recent_entities.retain(|_, entities| !entities.is_empty());

        // 检查最后操作的实体是否过期
        if let Some(ref entity) = self.last_operated_entity {
            if entity.is_stale(current_turn, max_turns) {
                self.last_operated_entity = None;
            }
        }
    }

    /// 解析代词，返回对应的实体引用
    ///
    /// 支持的代词：
    /// - "它"、"这个"、"那个" → last_operated_entity
    /// - "那个任务"、"刚才的任务"、"上一个任务" → recent_entities[Task].last()
    /// - "这个番茄钟"、"当前番茄钟" → recent_entities[Pomodoro].last()
    /// - "第一个"、"第二个" → recent_entities 中对应索引
    pub fn resolve_pronoun(&self, text: &str) -> Option<EntityReference> {
        let text_lower = text.to_lowercase();

        // 简单代词："它"、"这个"、"那个"
        if matches!(text_lower.as_str(), "它" | "这个" | "那个") {
            return self.last_operated_entity.clone();
        }

        // 带实体类型的代词
        if text_lower.contains("任务") {
            return self.get_latest_entity(EntityType::Task);
        }

        if text_lower.contains("番茄钟") || text_lower.contains("专注") {
            return self.get_latest_entity(EntityType::Pomodoro);
        }

        if text_lower.contains("项目") {
            return self.get_latest_entity(EntityType::Project);
        }

        if text_lower.contains("标签") {
            return self.get_latest_entity(EntityType::Tag);
        }

        if text_lower.contains("sql") || text_lower.contains("记录") {
            return self.get_latest_entity(EntityType::SqlRecord);
        }

        if text_lower.contains("报告") || text_lower.contains("日报") {
            return self.get_latest_entity(EntityType::Report);
        }

        // 数字引用："第一个"、"第二个"、"最后一个"
        if let Some(entity) = self.resolve_numeric_reference(&text_lower) {
            return Some(entity);
        }

        None
    }

    /// 解析数字引用（第一个、第二个、最后一个等）
    fn resolve_numeric_reference(&self, text: &str) -> Option<EntityReference> {
        // 确定实体类型
        let entity_type = if text.contains("任务") {
            Some(EntityType::Task)
        } else if text.contains("番茄钟") {
            Some(EntityType::Pomodoro)
        } else if text.contains("项目") {
            Some(EntityType::Project)
        } else if text.contains("标签") {
            Some(EntityType::Tag)
        } else {
            // 如果没有明确类型，尝试从最后操作的实体推断
            self.last_operated_entity.as_ref().map(|e| e.entity_type)
        };

        if let Some(entity_type) = entity_type {
            if let Some(entities) = self.recent_entities.get(&entity_type) {
                if entities.is_empty() {
                    return None;
                }

                // "最后一个"、"上一个"
                if text.contains("最后") || text.contains("上一") {
                    return entities.last().cloned();
                }

                // "第一个"
                if text.contains("第一") || text.contains("首个") {
                    return entities.first().cloned();
                }

                // "第二个"
                if text.contains("第二") {
                    return entities.get(1).cloned();
                }

                // "第三个"
                if text.contains("第三") {
                    return entities.get(2).cloned();
                }
            }
        }

        None
    }

    /// 获取指定类型的最新实体
    pub fn get_latest_entity(&self, entity_type: EntityType) -> Option<EntityReference> {
        self.recent_entities
            .get(&entity_type)
            .and_then(|entities| entities.last().cloned())
    }

    /// 获取指定类型的所有实体
    pub fn get_entities(&self, entity_type: EntityType) -> Vec<EntityReference> {
        self.recent_entities
            .get(&entity_type)
            .cloned()
            .unwrap_or_default()
    }

    /// 根据ID查找实体
    pub fn find_entity_by_id(&self, entity_type: EntityType, id: i64) -> Option<EntityReference> {
        self.recent_entities
            .get(&entity_type)?
            .iter()
            .find(|e| e.id == Some(id))
            .cloned()
    }

    /// 根据名称查找实体（模糊匹配）
    pub fn find_entity_by_name(&self, entity_type: EntityType, name: &str) -> Option<EntityReference> {
        let name_lower = name.to_lowercase();
        self.recent_entities
            .get(&entity_type)?
            .iter()
            .find(|e| e.name.to_lowercase().contains(&name_lower))
            .cloned()
    }

    /// 生成上下文提示（用于系统提示词）
    ///
    /// 返回一段描述当前对话上下文的文本，可以插入到AI提示词中
    pub fn get_context_hint(&self) -> String {
        let mut hints = Vec::new();

        // 当前主题
        if let Some(topic) = self.current_topic {
            let topic_str = match topic {
                ConversationTopic::TaskManagement => "任务管理",
                ConversationTopic::PomodoroFocus => "番茄钟专注",
                ConversationTopic::SqlSearch => "SQL搜索",
                ConversationTopic::ReportGeneration => "报告生成",
                ConversationTopic::GeneralChat => "通用对话",
            };
            hints.push(format!("当前对话主题：{}", topic_str));
        }

        // 最后操作的实体
        if let Some(ref entity) = self.last_operated_entity {
            let entity_type_str = entity_type_to_string(entity.entity_type);
            hints.push(format!(
                "最后操作：{} \"{}\"{}",
                entity_type_str,
                entity.name,
                entity.id.map(|id| format!(" (ID: {})", id)).unwrap_or_default()
            ));
        }

        // 最近提及的实体摘要
        for (entity_type, entities) in &self.recent_entities {
            if entities.is_empty() {
                continue;
            }
            let entity_type_str = entity_type_to_string(*entity_type);
            let count = entities.len();
            let latest = &entities[entities.len() - 1];
            hints.push(format!(
                "最近的{}：\"{}\" (共{}个)",
                entity_type_str,
                latest.name,
                count
            ));
        }

        if hints.is_empty() {
            "当前没有活跃的对话上下文。".to_string()
        } else {
            format!("对话上下文：\n{}", hints.join("\n"))
        }
    }

    /// 清空所有上下文
    pub fn clear(&mut self) {
        self.recent_entities.clear();
        self.current_topic = None;
        self.last_operated_entity = None;
        self.turn_count = 0;
    }

    /// 重置到新对话
    pub fn reset(&mut self) {
        self.clear();
    }

    /// 获取上下文统计信息
    pub fn get_stats(&self) -> ContextStats {
        let total_entities: usize = self.recent_entities.values().map(|v| v.len()).sum();
        let entity_counts: HashMap<EntityType, usize> = self
            .recent_entities
            .iter()
            .map(|(k, v)| (*k, v.len()))
            .collect();

        ContextStats {
            turn_count: self.turn_count,
            total_entities,
            entity_counts,
            current_topic: self.current_topic,
            has_last_operated: self.last_operated_entity.is_some(),
        }
    }
}

/// 上下文统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextStats {
    /// 对话轮次
    pub turn_count: usize,
    /// 总实体数
    pub total_entities: usize,
    /// 各类型实体数量
    pub entity_counts: HashMap<EntityType, usize>,
    /// 当前主题
    pub current_topic: Option<ConversationTopic>,
    /// 是否有最后操作的实体
    pub has_last_operated: bool,
}

/// 实体类型转字符串
fn entity_type_to_string(entity_type: EntityType) -> &'static str {
    match entity_type {
        EntityType::Task => "任务",
        EntityType::Pomodoro => "番茄钟",
        EntityType::Project => "项目",
        EntityType::Tag => "标签",
        EntityType::SqlRecord => "SQL记录",
        EntityType::Report => "报告",
    }
}

/// 对话上下文服务
pub struct ConversationContextService {
    /// 当前会话的上下文
    context: ConversationContext,
}

impl ConversationContextService {
    /// 创建新的服务实例
    pub fn new() -> Self {
        Self {
            context: ConversationContext::new(),
        }
    }

    /// 创建带自定义配置的服务实例
    pub fn with_max_turns(max_turns: usize) -> Self {
        Self {
            context: ConversationContext::with_max_turns(max_turns),
        }
    }

    /// 获取上下文引用
    pub fn context(&self) -> &ConversationContext {
        &self.context
    }

    /// 获取可变上下文引用
    pub fn context_mut(&mut self) -> &mut ConversationContext {
        &mut self.context
    }

    /// 处理用户消息，更新上下文
    pub fn process_message(&mut self, _message: &str) -> Result<()> {
        // 进入下一轮对话
        self.context.next_turn();

        // 这里可以添加更复杂的消息分析逻辑
        // 例如：NER实体识别、主题分类等

        Ok(())
    }

    /// 添加操作结果到上下文
    pub fn add_operation_result(
        &mut self,
        entity_type: EntityType,
        id: Option<i64>,
        name: String,
        metadata: Option<HashMap<String, String>>,
    ) {
        let entity = if let Some(metadata) = metadata {
            EntityReference::with_metadata(
                entity_type,
                id,
                name,
                self.context.turn_count,
                metadata,
            )
        } else {
            EntityReference::new(entity_type, id, name, self.context.turn_count)
        };

        self.context.set_last_operated(entity);
    }

    /// 解析用户输入中的代词引用
    pub fn resolve_reference(&self, text: &str) -> Option<EntityReference> {
        self.context.resolve_pronoun(text)
    }

    /// 生成上下文提示
    pub fn generate_context_hint(&self) -> String {
        self.context.get_context_hint()
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> ContextStats {
        self.context.get_stats()
    }

    /// 重置上下文
    pub fn reset(&mut self) {
        self.context.reset();
    }
}

impl Default for ConversationContextService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_entity_tracking() {
        let mut context = ConversationContext::new();

        // 添加任务
        let task = EntityReference::new(
            EntityType::Task,
            Some(1),
            "完成项目文档".to_string(),
            context.turn_count,
        );
        context.add_entity(task.clone());

        // 验证添加成功
        assert_eq!(context.get_entities(EntityType::Task).len(), 1);
        assert_eq!(
            context.get_latest_entity(EntityType::Task).unwrap().name,
            "完成项目文档"
        );
    }

    #[test]
    fn test_pronoun_resolution() {
        let mut context = ConversationContext::new();

        // 添加并操作任务
        let task = EntityReference::new(
            EntityType::Task,
            Some(1),
            "测试任务".to_string(),
            context.turn_count,
        );
        context.set_last_operated(task);

        // 测试代词解析
        assert!(context.resolve_pronoun("它").is_some());
        assert!(context.resolve_pronoun("这个").is_some());
        assert!(context.resolve_pronoun("那个任务").is_some());

        let resolved = context.resolve_pronoun("它").unwrap();
        assert_eq!(resolved.name, "测试任务");
    }

    #[test]
    fn test_stale_entity_cleanup() {
        let mut context = ConversationContext::with_max_turns(2);

        // 添加实体
        let task1 = EntityReference::new(
            EntityType::Task,
            Some(1),
            "任务1".to_string(),
            0,
        );
        context.add_entity(task1);

        // 前进轮次
        context.next_turn(); // turn 1
        context.next_turn(); // turn 2
        context.next_turn(); // turn 3 - 应该清理turn 0的实体

        // 验证实体已被清理
        assert_eq!(context.get_entities(EntityType::Task).len(), 0);
    }

    #[test]
    fn test_numeric_reference() {
        let mut context = ConversationContext::new();

        // 添加多个任务
        for i in 1..=3 {
            let task = EntityReference::new(
                EntityType::Task,
                Some(i),
                format!("任务{}", i),
                context.turn_count,
            );
            context.add_entity(task);
        }

        // 测试数字引用
        assert_eq!(
            context.resolve_pronoun("第一个任务").unwrap().name,
            "任务1"
        );
        assert_eq!(
            context.resolve_pronoun("第二个任务").unwrap().name,
            "任务2"
        );
        assert_eq!(
            context.resolve_pronoun("最后一个任务").unwrap().name,
            "任务3"
        );
    }

    #[test]
    fn test_context_hint_generation() {
        let mut context = ConversationContext::new();

        // 设置主题
        context.update_topic(ConversationTopic::TaskManagement);

        // 添加实体
        let task = EntityReference::new(
            EntityType::Task,
            Some(1),
            "重要任务".to_string(),
            context.turn_count,
        );
        context.set_last_operated(task);

        // 生成提示
        let hint = context.get_context_hint();
        assert!(hint.contains("任务管理"));
        assert!(hint.contains("重要任务"));
    }

    #[test]
    fn test_entity_deduplication() {
        let mut context = ConversationContext::new();

        // 多次添加相同ID的实体
        for i in 0..3 {
            let task = EntityReference::new(
                EntityType::Task,
                Some(1),
                format!("任务名称{}", i),
                context.turn_count,
            );
            context.add_entity(task);
        }

        // 验证只保留一个，且名称为最新的
        let tasks = context.get_entities(EntityType::Task);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name, "任务名称2");
    }

    #[test]
    fn test_service_workflow() {
        let mut service = ConversationContextService::new();

        // 处理消息
        service.process_message("创建一个任务").unwrap();

        // 添加操作结果
        service.add_operation_result(
            EntityType::Task,
            Some(1),
            "新任务".to_string(),
            None,
        );

        // 验证可以解析引用
        let resolved = service.resolve_reference("它");
        assert!(resolved.is_some());
        assert_eq!(resolved.unwrap().name, "新任务");

        // 获取统计
        let stats = service.get_stats();
        assert_eq!(stats.turn_count, 1);
        assert_eq!(stats.total_entities, 1);
    }
}
