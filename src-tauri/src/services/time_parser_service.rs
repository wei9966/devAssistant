use chrono::{NaiveDate, Datelike, Duration, Weekday, Local};
use regex::Regex;
use std::sync::OnceLock;

#[derive(Debug, Clone)]
pub struct TimeExpression {
    pub original: String,      // 原始文本
    pub parsed_date: NaiveDate, // 解析后的日期
    pub confidence: f32,       // 置信度 0-1
}

pub struct TimeParserService;

impl TimeParserService {
    /// 解析自然语言时间表达，返回日期
    pub fn parse_natural_time(text: &str) -> Option<NaiveDate> {
        let today = Local::now().date_naive();
        let text = text.trim();

        // 相对日期
        if let Some(date) = Self::parse_relative_day(text, today) {
            return Some(date);
        }

        // 相对周
        if let Some(date) = Self::parse_relative_week(text, today) {
            return Some(date);
        }

        // 相对时间（天/周/月/年后）
        if let Some(date) = Self::parse_relative_time(text, today) {
            return Some(date);
        }

        // 具体日期
        if let Some(date) = Self::parse_specific_date(text, today) {
            return Some(date);
        }

        // 模糊表达
        if let Some(date) = Self::parse_fuzzy_date(text, today) {
            return Some(date);
        }

        // 特殊表达
        if let Some(date) = Self::parse_special_expression(text, today) {
            return Some(date);
        }

        None
    }

    /// 从文本中提取所有时间表达
    pub fn extract_time_expressions(text: &str) -> Vec<TimeExpression> {
        let mut expressions = Vec::new();
        let patterns = Self::get_time_patterns();

        for pattern in patterns {
            let re = Regex::new(pattern).unwrap();
            for cap in re.captures_iter(text) {
                if let Some(matched) = cap.get(0) {
                    let original = matched.as_str().to_string();
                    if let Some(parsed_date) = Self::parse_natural_time(&original) {
                        let confidence = Self::calculate_confidence(&original);
                        expressions.push(TimeExpression {
                            original,
                            parsed_date,
                            confidence,
                        });
                    }
                }
            }
        }

        expressions
    }

    /// 解析相对日期（今天、明天、后天等）
    fn parse_relative_day(text: &str, today: NaiveDate) -> Option<NaiveDate> {
        match text {
            "今天" | "今日" => Some(today),
            "明天" | "明日" => Some(today + Duration::days(1)),
            "后天" => Some(today + Duration::days(2)),
            "大后天" => Some(today + Duration::days(3)),
            "昨天" | "昨日" => Some(today - Duration::days(1)),
            "前天" => Some(today - Duration::days(2)),
            "大前天" => Some(today - Duration::days(3)),
            _ => None,
        }
    }

    /// 解析相对周（下周一、这周日等）
    fn parse_relative_week(text: &str, today: NaiveDate) -> Option<NaiveDate> {
        static RE: OnceLock<Regex> = OnceLock::new();
        let re = RE.get_or_init(|| {
            Regex::new(r"^(上上|下下|上|下|这|本)周([一二三四五六日天])$").unwrap()
        });

        if let Some(caps) = re.captures(text) {
            let prefix = caps.get(1)?.as_str();
            let weekday_str = caps.get(2)?.as_str();

            let target_weekday = match weekday_str {
                "一" => Weekday::Mon,
                "二" => Weekday::Tue,
                "三" => Weekday::Wed,
                "四" => Weekday::Thu,
                "五" => Weekday::Fri,
                "六" => Weekday::Sat,
                "日" | "天" => Weekday::Sun,
                _ => return None,
            };

            let current_weekday = today.weekday();
            let days_until_target = (target_weekday.num_days_from_monday() as i64
                - current_weekday.num_days_from_monday() as i64 + 7) % 7;

            let weeks_offset = match prefix {
                "上上" => -2,
                "上" => -1,
                "这" | "本" => 0,
                "下" => 1,
                "下下" => 2,
                _ => return None,
            };

            let mut target_date = today + Duration::days(days_until_target);
            if weeks_offset != 0 {
                if days_until_target == 0 && weeks_offset > 0 {
                    // 如果是同一天且要找未来的周，加一周
                    target_date = target_date + Duration::weeks(weeks_offset as i64);
                } else if days_until_target == 0 && weeks_offset < 0 {
                    // 如果是同一天且要找过去的周，不需要额外处理
                    target_date = target_date + Duration::weeks(weeks_offset as i64);
                } else {
                    target_date = target_date + Duration::weeks(weeks_offset as i64);
                }
            }

            return Some(target_date);
        }

        None
    }

    /// 解析相对时间（3天后、一周后等）
    fn parse_relative_time(text: &str, today: NaiveDate) -> Option<NaiveDate> {
        // 匹配 "N天/周/月/年后"
        static RE_DAYS: OnceLock<Regex> = OnceLock::new();
        let re_days = RE_DAYS.get_or_init(|| {
            Regex::new(r"^([0-9零一二三四五六七八九十百千万]+)(天|日)([后前]|以[后前])$").unwrap()
        });

        if let Some(caps) = re_days.captures(text) {
            let num_str = caps.get(1)?.as_str();
            let direction = caps.get(3)?.as_str();
            let num = Self::chinese_to_number(num_str)?;

            return Some(if direction.contains("后") {
                today + Duration::days(num)
            } else {
                today - Duration::days(num)
            });
        }

        // 匹配 "N周后"
        static RE_WEEKS: OnceLock<Regex> = OnceLock::new();
        let re_weeks = RE_WEEKS.get_or_init(|| {
            Regex::new(r"^([0-9零一二三四五六七八九十]+)(周|星期)([后前]|以[后前])$").unwrap()
        });

        if let Some(caps) = re_weeks.captures(text) {
            let num_str = caps.get(1)?.as_str();
            let direction = caps.get(3)?.as_str();
            let num = Self::chinese_to_number(num_str)?;

            return Some(if direction.contains("后") {
                today + Duration::weeks(num)
            } else {
                today - Duration::weeks(num)
            });
        }

        // 匹配 "N个月后"
        static RE_MONTHS: OnceLock<Regex> = OnceLock::new();
        let re_months = RE_MONTHS.get_or_init(|| {
            Regex::new(r"^([0-9零一二三四五六七八九十]+)个?月([后前]|以[后前])$").unwrap()
        });

        if let Some(caps) = re_months.captures(text) {
            let num_str = caps.get(1)?.as_str();
            let direction = caps.get(2)?.as_str();
            let num = Self::chinese_to_number(num_str)?;

            let (year, month) = if direction.contains("后") {
                let total_months = today.year() * 12 + today.month() as i32 + num as i32;
                ((total_months - 1) / 12, (total_months - 1) % 12 + 1)
            } else {
                let total_months = today.year() * 12 + today.month() as i32 - num as i32;
                ((total_months - 1) / 12, (total_months - 1) % 12 + 1)
            };

            let day = today.day().min(Self::days_in_month(year, month as u32));
            return NaiveDate::from_ymd_opt(year, month as u32, day);
        }

        // 匹配 "半年后"
        if text.contains("半年") {
            let direction = if text.contains("后") { 6 } else { -6 };
            let total_months = today.year() * 12 + today.month() as i32 + direction;
            let year = (total_months - 1) / 12;
            let month = (total_months - 1) % 12 + 1;
            let day = today.day().min(Self::days_in_month(year, month as u32));
            return NaiveDate::from_ymd_opt(year, month as u32, day);
        }

        // 匹配 "N年后"
        static RE_YEARS: OnceLock<Regex> = OnceLock::new();
        let re_years = RE_YEARS.get_or_init(|| {
            Regex::new(r"^([0-9零一二三四五六七八九十]+)年([后前]|以[后前])$").unwrap()
        });

        if let Some(caps) = re_years.captures(text) {
            let num_str = caps.get(1)?.as_str();
            let direction = caps.get(2)?.as_str();
            let num = Self::chinese_to_number(num_str)?;

            let year = if direction.contains("后") {
                today.year() + num as i32
            } else {
                today.year() - num as i32
            };

            let day = today.day().min(Self::days_in_month(year, today.month()));
            return NaiveDate::from_ymd_opt(year, today.month(), day);
        }

        None
    }

    /// 解析具体日期（1月15号、下个月5号等）
    fn parse_specific_date(text: &str, today: NaiveDate) -> Option<NaiveDate> {
        // 匹配 "M月D号/日"
        static RE_MD: OnceLock<Regex> = OnceLock::new();
        let re_md = RE_MD.get_or_init(|| {
            Regex::new(r"^([0-9]{1,2}|十?[一二三四五六七八九十]+)月([0-9]{1,2}|[一二三四五六七八九十]+)(号|日)$").unwrap()
        });

        if let Some(caps) = re_md.captures(text) {
            let month_str = caps.get(1)?.as_str();
            let day_str = caps.get(2)?.as_str();

            let month = Self::parse_month_or_day(month_str)?;
            let day = Self::parse_month_or_day(day_str)?;

            if month > 0 && month <= 12 && day > 0 && day <= 31 {
                let mut year = today.year();
                // 如果日期已过，使用明年
                if month < today.month() as i64 ||
                   (month == today.month() as i64 && day < today.day() as i64) {
                    year += 1;
                }
                return NaiveDate::from_ymd_opt(year, month as u32, day as u32);
            }
        }

        // 匹配 "下/上个月D号"
        static RE_NEXT_MONTH: OnceLock<Regex> = OnceLock::new();
        let re_next_month = RE_NEXT_MONTH.get_or_init(|| {
            Regex::new(r"^(下|上)个?月([0-9]{1,2}|[一二三四五六七八九十]+)(号|日)$").unwrap()
        });

        if let Some(caps) = re_next_month.captures(text) {
            let direction = caps.get(1)?.as_str();
            let day_str = caps.get(2)?.as_str();
            let day = Self::parse_month_or_day(day_str)?;

            if day > 0 && day <= 31 {
                let offset = if direction == "下" { 1 } else { -1 };
                let total_months = today.year() * 12 + today.month() as i32 + offset;
                let year = (total_months - 1) / 12;
                let month = (total_months - 1) % 12 + 1;
                let actual_day = day.min(Self::days_in_month(year, month as u32) as i64);
                return NaiveDate::from_ymd_opt(year, month as u32, actual_day as u32);
            }
        }

        None
    }

    /// 解析模糊日期（月底、年底等）
    fn parse_fuzzy_date(text: &str, today: NaiveDate) -> Option<NaiveDate> {
        match text {
            "月初" => {
                NaiveDate::from_ymd_opt(today.year(), today.month(), 1)
            }
            "月底" | "月末" => {
                let days = Self::days_in_month(today.year(), today.month());
                NaiveDate::from_ymd_opt(today.year(), today.month(), days)
            }
            "下月初" | "下个月初" => {
                let total_months = today.year() * 12 + today.month() as i32 + 1;
                let year = (total_months - 1) / 12;
                let month = (total_months - 1) % 12 + 1;
                NaiveDate::from_ymd_opt(year, month as u32, 1)
            }
            "下月底" | "下个月底" | "下月末" | "下个月末" => {
                let total_months = today.year() * 12 + today.month() as i32 + 1;
                let year = (total_months - 1) / 12;
                let month = (total_months - 1) % 12 + 1;
                let days = Self::days_in_month(year, month as u32);
                NaiveDate::from_ymd_opt(year, month as u32, days)
            }
            "年初" => {
                NaiveDate::from_ymd_opt(today.year(), 1, 1)
            }
            "年底" | "年末" => {
                NaiveDate::from_ymd_opt(today.year(), 12, 31)
            }
            "明年初" | "下年初" => {
                NaiveDate::from_ymd_opt(today.year() + 1, 1, 1)
            }
            "明年底" | "明年末" | "下年底" | "下年末" => {
                NaiveDate::from_ymd_opt(today.year() + 1, 12, 31)
            }
            _ => None,
        }
    }

    /// 解析特殊表达（周末、下个工作日等）
    fn parse_special_expression(text: &str, today: NaiveDate) -> Option<NaiveDate> {
        match text {
            "周末" | "本周末" | "这周末" => {
                let current_weekday = today.weekday();
                let days_until_sat = (Weekday::Sat.num_days_from_monday() as i64
                    - current_weekday.num_days_from_monday() as i64 + 7) % 7;
                let saturday = if days_until_sat == 0 {
                    today
                } else {
                    today + Duration::days(days_until_sat)
                };
                Some(saturday)
            }
            "下周末" => {
                let current_weekday = today.weekday();
                let days_until_sat = (Weekday::Sat.num_days_from_monday() as i64
                    - current_weekday.num_days_from_monday() as i64 + 7) % 7;
                let this_saturday = today + Duration::days(days_until_sat);
                Some(this_saturday + Duration::weeks(1))
            }
            "下个工作日" | "下一个工作日" => {
                let mut date = today + Duration::days(1);
                while date.weekday() == Weekday::Sat || date.weekday() == Weekday::Sun {
                    date = date + Duration::days(1);
                }
                Some(date)
            }
            "上个工作日" | "上一个工作日" => {
                let mut date = today - Duration::days(1);
                while date.weekday() == Weekday::Sat || date.weekday() == Weekday::Sun {
                    date = date - Duration::days(1);
                }
                Some(date)
            }
            _ => None,
        }
    }

    /// 获取时间表达的正则模式
    fn get_time_patterns() -> Vec<&'static str> {
        vec![
            // 相对日期
            r"(今天|今日|明天|明日|后天|大后天|昨天|昨日|前天|大前天)",
            // 相对周
            r"(上上|下下|上|下|这|本)周[一二三四五六日天]",
            // 相对时间
            r"[0-9零一二三四五六七八九十百千万]+(天|日|周|星期|个?月|年)(后|前|以后|以前)",
            r"半年(后|前|以后|以前)",
            // 具体日期
            r"([0-9]{1,2}|十?[一二三四五六七八九十]+)月([0-9]{1,2}|[一二三四五六七八九十]+)(号|日)",
            r"(下|上)个?月([0-9]{1,2}|[一二三四五六七八九十]+)(号|日)",
            // 模糊表达
            r"(月初|月底|月末|下月初|下个月初|下月底|下个月底|下月末|下个月末|年初|年底|年末|明年初|下年初|明年底|明年末|下年底|下年末)",
            // 特殊表达
            r"(周末|本周末|这周末|下周末|下个工作日|下一个工作日|上个工作日|上一个工作日)",
        ]
    }

    /// 计算置信度
    fn calculate_confidence(text: &str) -> f32 {
        // 简单规则：越具体的表达置信度越高
        if text.chars().any(|c| c.is_ascii_digit()) {
            0.95 // 包含数字的表达
        } else if text.contains("今天") || text.contains("明天") {
            0.99 // 明确的相对日期
        } else if text.contains("周") {
            0.90 // 周相关
        } else if text.contains("月") || text.contains("年") {
            0.85 // 月年相关
        } else {
            0.80 // 其他表达
        }
    }

    /// 中文数字转阿拉伯数字
    fn chinese_to_number(text: &str) -> Option<i64> {
        // 如果是阿拉伯数字，直接解析
        if let Ok(num) = text.parse::<i64>() {
            return Some(num);
        }

        // 中文数字映射
        let mut result = 0i64;
        let mut current = 0i64;
        let mut has_unit = false;

        for ch in text.chars() {
            match ch {
                '零' => {}
                '一' => current = 1,
                '二' => current = 2,
                '三' => current = 3,
                '四' => current = 4,
                '五' => current = 5,
                '六' => current = 6,
                '七' => current = 7,
                '八' => current = 8,
                '九' => current = 9,
                '十' => {
                    has_unit = true;
                    if current == 0 {
                        current = 1;
                    }
                    result += current * 10;
                    current = 0;
                }
                '百' => {
                    has_unit = true;
                    if current == 0 {
                        current = 1;
                    }
                    result += current * 100;
                    current = 0;
                }
                '千' => {
                    has_unit = true;
                    if current == 0 {
                        current = 1;
                    }
                    result += current * 1000;
                    current = 0;
                }
                '万' => {
                    has_unit = true;
                    if current == 0 && result == 0 {
                        current = 1;
                    }
                    result = (result + current) * 10000;
                    current = 0;
                }
                _ => return None,
            }
        }

        result += current;

        // 如果没有遇到单位词且结果为0，说明只有个位数
        if !has_unit && result == 0 {
            result = current;
        }

        Some(result)
    }

    /// 解析月份或日期数字（支持中文和阿拉伯数字）
    fn parse_month_or_day(text: &str) -> Option<i64> {
        Self::chinese_to_number(text)
    }

    /// 获取某月的天数
    fn days_in_month(year: i32, month: u32) -> u32 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if Self::is_leap_year(year) { 29 } else { 28 }
            }
            _ => 30,
        }
    }

    /// 判断是否为闰年
    fn is_leap_year(year: i32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relative_day() {
        let today = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();

        assert_eq!(
            TimeParserService::parse_relative_day("今天", today),
            Some(NaiveDate::from_ymd_opt(2025, 1, 15).unwrap())
        );
        assert_eq!(
            TimeParserService::parse_relative_day("明天", today),
            Some(NaiveDate::from_ymd_opt(2025, 1, 16).unwrap())
        );
        assert_eq!(
            TimeParserService::parse_relative_day("后天", today),
            Some(NaiveDate::from_ymd_opt(2025, 1, 17).unwrap())
        );
    }

    #[test]
    fn test_chinese_to_number() {
        assert_eq!(TimeParserService::chinese_to_number("一"), Some(1));
        assert_eq!(TimeParserService::chinese_to_number("十"), Some(10));
        assert_eq!(TimeParserService::chinese_to_number("十五"), Some(15));
        assert_eq!(TimeParserService::chinese_to_number("二十"), Some(20));
        assert_eq!(TimeParserService::chinese_to_number("三十一"), Some(31));
        assert_eq!(TimeParserService::chinese_to_number("一百"), Some(100));
    }

    #[test]
    fn test_extract_time_expressions() {
        let text = "记得明天下午3点开会，后天要提交报告";
        let expressions = TimeParserService::extract_time_expressions(text);

        assert!(expressions.len() >= 2);
        assert!(expressions.iter().any(|e| e.original.contains("明天")));
        assert!(expressions.iter().any(|e| e.original.contains("后天")));
    }
}
