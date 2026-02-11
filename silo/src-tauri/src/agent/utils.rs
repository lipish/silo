// Agent 工具函数

/// 从文本中提取关键词
pub fn extract_keywords(text: &str) -> String {
    let stop_words = [
        "的", "了", "在", "是", "我", "有", "和", "就", "不", "人", "都", "一", "一个",
        "上", "也", "很", "到", "说", "要", "去", "你", "会", "着", "没有", "看", "好",
        "自己", "这", "the", "a", "an", "is", "are", "was", "were", "be", "been", "being",
        "have", "has", "had", "do", "does", "did", "will", "would", "should", "could",
    ];
    
    text.split_whitespace()
        .filter(|w| {
            let w_lower = w.to_lowercase();
            !stop_words.contains(&w_lower.as_str()) && w.len() > 1
        })
        .take(5)
        .collect::<Vec<_>>()
        .join(" ")
}

/// 从指令中提取搜索查询
pub fn extract_search_query(instruction: &str) -> String {
    // 简单的关键词提取
    extract_keywords(instruction)
}

/// 从文本中提取代码块
pub fn extract_code_block(text: &str) -> Option<String> {
    // 查找代码块标记
    if let Some(start) = text.find("```") {
        if let Some(end) = text[start + 3..].find("```") {
            let code = &text[start + 3..start + 3 + end];
            // 移除语言标识
            let code = code.lines()
                .skip(1) // 跳过第一行（可能是语言标识）
                .collect::<Vec<_>>()
                .join("\n");
            return Some(code.trim().to_string());
        }
    }
    None
}
