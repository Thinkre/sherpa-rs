use log::{debug, warn};
use natural::phonetics::soundex;
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use strsim::levenshtein;

/// Applies custom word corrections to transcribed text using fuzzy matching
///
/// This function corrects words in the input text by finding the best matches
/// from a list of custom words using a combination of:
/// - Levenshtein distance for string similarity
/// - Soundex phonetic matching for pronunciation similarity
///
/// # Arguments
/// * `text` - The input text to correct
/// * `custom_words` - List of custom words to match against
/// * `threshold` - Maximum similarity score to accept (0.0 = exact match, 1.0 = any match)
///
/// # Returns
/// The corrected text with custom words applied
pub fn apply_custom_words(text: &str, custom_words: &[String], threshold: f64) -> String {
    if custom_words.is_empty() {
        return text.to_string();
    }

    // Pre-compute lowercase versions to avoid repeated allocations
    let custom_words_lower: Vec<String> = custom_words.iter().map(|w| w.to_lowercase()).collect();

    let words: Vec<&str> = text.split_whitespace().collect();
    let mut corrected_words = Vec::new();

    for word in words {
        let cleaned_word = word
            .trim_matches(|c: char| !c.is_alphabetic())
            .to_lowercase();

        if cleaned_word.is_empty() {
            corrected_words.push(word.to_string());
            continue;
        }

        // Skip extremely long words to avoid performance issues
        if cleaned_word.len() > 50 {
            corrected_words.push(word.to_string());
            continue;
        }

        let mut best_match: Option<&String> = None;
        let mut best_score = f64::MAX;

        for (i, custom_word_lower) in custom_words_lower.iter().enumerate() {
            // Skip if lengths are too different (optimization)
            let len_diff = (cleaned_word.len() as i32 - custom_word_lower.len() as i32).abs();
            if len_diff > 5 {
                continue;
            }

            // Calculate Levenshtein distance (normalized by length)
            let levenshtein_dist = levenshtein(&cleaned_word, custom_word_lower);
            let max_len = cleaned_word.len().max(custom_word_lower.len()) as f64;
            let levenshtein_score = if max_len > 0.0 {
                levenshtein_dist as f64 / max_len
            } else {
                1.0
            };

            // Calculate phonetic similarity using Soundex
            let phonetic_match = soundex(&cleaned_word, custom_word_lower);

            // Combine scores: favor phonetic matches, but also consider string similarity
            let combined_score = if phonetic_match {
                levenshtein_score * 0.3 // Give significant boost to phonetic matches
            } else {
                levenshtein_score
            };

            // Accept if the score is good enough (configurable threshold)
            if combined_score < threshold && combined_score < best_score {
                best_match = Some(&custom_words[i]);
                best_score = combined_score;
            }
        }

        if let Some(replacement) = best_match {
            // Preserve the original case pattern as much as possible
            let corrected = preserve_case_pattern(word, replacement);

            // Preserve punctuation from original word
            let (prefix, suffix) = extract_punctuation(word);
            corrected_words.push(format!("{}{}{}", prefix, corrected, suffix));
        } else {
            corrected_words.push(word.to_string());
        }
    }

    corrected_words.join(" ")
}

/// Preserves the case pattern of the original word when applying a replacement
fn preserve_case_pattern(original: &str, replacement: &str) -> String {
    if original.chars().all(|c| c.is_uppercase()) {
        replacement.to_uppercase()
    } else if original.chars().next().map_or(false, |c| c.is_uppercase()) {
        let mut chars: Vec<char> = replacement.chars().collect();
        if let Some(first_char) = chars.get_mut(0) {
            *first_char = first_char.to_uppercase().next().unwrap_or(*first_char);
        }
        chars.into_iter().collect()
    } else {
        replacement.to_string()
    }
}

/// Extracts punctuation prefix and suffix from a word
fn extract_punctuation(word: &str) -> (String, String) {
    let prefix_end_char = word.chars().take_while(|c| !c.is_alphabetic()).count();
    let suffix_start_char = word
        .chars()
        .rev()
        .take_while(|c| !c.is_alphabetic())
        .count();

    let prefix = if prefix_end_char > 0 {
        word.chars().take(prefix_end_char).collect::<String>()
    } else {
        String::new()
    };

    let suffix = if suffix_start_char > 0 {
        word.chars().rev().take(suffix_start_char).collect::<String>().chars().rev().collect::<String>()
    } else {
        String::new()
    };

    (prefix, suffix)
}

/// Filler words to remove from transcriptions
const FILLER_WORDS: &[&str] = &[
    "uh", "um", "uhm", "umm", "uhh", "uhhh", "ah", "eh", "hmm", "hm", "mmm", "mm", "mh", "ha",
    "ehh",
];

static MULTI_SPACE_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s{2,}").unwrap());

/// Collapses repeated 1-2 letter words (3+ repetitions) to a single instance.
/// E.g., "wh wh wh wh" -> "wh", "I I I I" -> "I"
fn collapse_stutters(text: &str) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return text.to_string();
    }

    let mut result: Vec<&str> = Vec::new();
    let mut i = 0;

    while i < words.len() {
        let word = words[i];
        let word_lower = word.to_lowercase();

        // Only process 1-2 letter words
        if word_lower.len() <= 2 && word_lower.chars().all(|c| c.is_alphabetic()) {
            // Count consecutive repetitions (case-insensitive)
            let mut count = 1;
            while i + count < words.len() && words[i + count].to_lowercase() == word_lower {
                count += 1;
            }

            // If 3+ repetitions, collapse to single instance
            if count >= 3 {
                result.push(word);
                i += count;
            } else {
                result.push(word);
                i += 1;
            }
        } else {
            result.push(word);
            i += 1;
        }
    }

    result.join(" ")
}

/// Pre-compiled filler word patterns (built lazily)
static FILLER_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    FILLER_WORDS
        .iter()
        .map(|word| {
            // Match filler word with word boundaries, optionally followed by comma or period
            Regex::new(&format!(r"(?i)\b{}\b[,.]?", regex::escape(word))).unwrap()
        })
        .collect()
});

/// Filters transcription output by removing filler words and stutter artifacts.
///
/// This function cleans up raw transcription text by:
/// 1. Removing filler words (uh, um, hmm, etc.)
/// 2. Collapsing repeated 1-2 letter stutters (e.g., "wh wh wh" -> "wh")
/// 3. Cleaning up excess whitespace
///
/// # Arguments
/// * `text` - The raw transcription text to filter
///
/// # Returns
/// The filtered text with filler words and stutters removed
pub fn filter_transcription_output(text: &str) -> String {
    let mut filtered = text.to_string();

    // Remove filler words
    for pattern in FILLER_PATTERNS.iter() {
        filtered = pattern.replace_all(&filtered, "").to_string();
    }

    // Collapse repeated 1-2 letter words (stutter artifacts like "wh wh wh wh")
    filtered = collapse_stutters(&filtered);

    // Clean up multiple spaces to single space
    filtered = MULTI_SPACE_PATTERN.replace_all(&filtered, " ").to_string();

    // Trim leading/trailing whitespace
    filtered.trim().to_string()
}

/// Chinese number to digit mapping
/// Enhanced with more variants based on WeTextProcessing and cn_tn.py
const CHINESE_NUMBERS: &[(&str, i64)] = &[
    ("零", 0),
    ("〇", 0), // Alternative zero character
    ("一", 1),
    ("幺", 1), // Alternative one (used in phone numbers)
    ("二", 2),
    ("两", 2), // Alternative two
    ("三", 3),
    ("四", 4),
    ("五", 5),
    ("六", 6),
    ("七", 7),
    ("八", 8),
    ("九", 9),
    ("十", 10),
    ("拾", 10), // Traditional form
    ("百", 100),
    ("佰", 100), // Traditional form
    ("千", 1000),
    ("仟", 1000), // Traditional form
    ("万", 10000),
    ("萬", 10000), // Traditional form
    ("亿", 100000000),
    ("億", 100000000), // Traditional form
];

/// Chinese number units for large numbers
const CHINESE_LARGE_UNITS: &[(&str, i64)] = &[
    ("兆", 1000000000000i64), // 1 trillion
    ("京", 10000000000000000i64), // 10 quadrillion
];

/// Chinese currency units
const CURRENCY_UNITS: &[&str] = &["元", "块", "角", "毛", "分"];

/// Chinese quantifiers (common measure words)
const QUANTIFIERS: &[&str] = &[
    "个", "人", "件", "次", "月", "日", "天", "小时", "分钟", "秒", "年", "周", "星期",
    "只", "条", "张", "本", "支", "把", "台", "辆", "头", "匹", "座", "栋", "间", "层",
    "斤", "两", "克", "千克", "吨", "米", "厘米", "毫米", "公里", "里", "尺", "寸",
];

/// Chinese location/place name suffixes (should not convert numbers before these)
const LOCATION_SUFFIXES: &[&str] = &[
    "街", "路", "巷", "道", "区", "旗", "村", "镇", "县", "市", "省", "州", "府", "县",
    "站", "桥", "门", "口", "楼", "层", "号", "弄", "里", "坊", "庄", "园", "苑", "居",
    "东", "西", "南", "北", "中", "上", "下", "前", "后", "左", "右",
];

/// Fixed phrases/idioms that should not have numbers converted
/// These are professional terms, idioms, or fixed expressions where numbers should remain in Chinese
const FIXED_PHRASES: &[&str] = &[
    "满五唯一",      // Real estate term: "满五年且唯一住房"
    "满二唯一",      // Real estate term: "满二年且唯一住房"
    "满五",          // Real estate term: "满五年"
    "满二",          // Real estate term: "满二年"
    "唯一",          // "唯一" (unique/only)
    "三心二意",      // Idiom: "half-hearted"
    "一心一意",      // Idiom: "wholeheartedly"
    "三三两两",      // Idiom: "in small groups"
    "一五一十",      // Idiom: "systematically"
    "一清二楚",      // Idiom: "crystal clear"
    "一石二鸟",      // Idiom: "kill two birds with one stone"
    "三思而后行",    // Idiom: "think twice before acting"
    "四面八方",      // Idiom: "all directions"
    "五湖四海",      // Idiom: "all corners of the country"
    "七上八下",      // Idiom: "anxious"
    "九牛一毛",      // Idiom: "a drop in the bucket"
    "十全十美",      // Idiom: "perfect"
];

/// Converts Chinese numbers to Arabic digits (ITN - Inverse Text Normalization)
/// Enhanced implementation based on WeTextProcessing and cn_tn.py
/// Supports various formats including dates, times, currency, percentages, phone numbers, etc.
pub fn apply_itn(text: &str) -> String {
    let original = text.to_string();
    let mut result = text.to_string();

    // #region agent log
    use std::fs::OpenOptions;
    use std::io::Write;
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/Users/thinkre/Desktop/projects/KeVoiceInput/.cursor/debug.log")
    {
        let _ = writeln!(
            file,
            r#"{{"id":"log_{}","timestamp":{},"location":"text.rs:253","message":"ITN input","data":{{"original":"{}"}},"sessionId":"debug-session","runId":"itn-debug","hypothesisId":"A"}}"#,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            original.replace('"', r#"\""#)
        );
    }
    // #endregion

    // First, protect fixed phrases/idioms from number conversion
    // Mark fixed phrases with special markers, then restore them at the end
    // Use a HashMap to store marker -> original phrase mapping
    let mut phrase_map: HashMap<String, String> = HashMap::new();
    let mut marker_counter = 0;
    
    for phrase in FIXED_PHRASES.iter() {
        let mut search_start = 0;
        while let Some(pos) = result[search_start..].find(phrase) {
            let actual_pos = search_start + pos;
            let marker = format!("__FIXED_PHRASE_{}__", marker_counter);
            marker_counter += 1;
            phrase_map.insert(marker.clone(), phrase.to_string());
            result.replace_range(actual_pos..actual_pos + phrase.len(), &marker);
            search_start = actual_pos + marker.len();
        }
    }

    // Create a mapping for quick lookup (including all variants)
    let mut num_map: HashMap<&str, i64> = CHINESE_NUMBERS.iter().cloned().collect();
    // Add large units
    for (unit, value) in CHINESE_LARGE_UNITS.iter() {
        num_map.insert(unit, *value);
    }

    // Pattern 0: Dates - "二零二四年" -> "2024年", "二零二四年一月一日" -> "2024年1月1日"
    // Handle full date format first
    let full_date_pattern = Regex::new(
        r"([零一二三四五六七八九〇]{4})年([零一二三四五六七八九十]+)月([零一二三四五六七八九十]+)[日号]"
    ).unwrap();
    result = full_date_pattern.replace_all(&result, |caps: &regex::Captures| {
        let year_str = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let month_str = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        let day_str = caps.get(3).map(|m| m.as_str()).unwrap_or("");
        
        let year = convert_year(year_str, &num_map);
        let month = parse_chinese_number(month_str, &num_map).unwrap_or(0);
        let day = parse_chinese_number(day_str, &num_map).unwrap_or(0);
        
        if year > 0 && month > 0 && day > 0 {
            format!("{}年{}月{}日", year, month, day)
        } else {
            caps.get(0).map(|m| m.as_str()).unwrap_or("").to_string()
        }
    }).to_string();

    // Handle year-month format: "二零二四年一月" -> "2024年1月"
    let year_month_pattern = Regex::new(
        r"([零一二三四五六七八九〇]{4})年([零一二三四五六七八九十]+)月"
    ).unwrap();
    result = year_month_pattern.replace_all(&result, |caps: &regex::Captures| {
        let year_str = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let month_str = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        
        let year = convert_year(year_str, &num_map);
        let month = parse_chinese_number(month_str, &num_map).unwrap_or(0);
        
        if year > 0 && month > 0 {
            format!("{}年{}月", year, month)
        } else {
            caps.get(0).map(|m| m.as_str()).unwrap_or("").to_string()
        }
    }).to_string();

    // Handle year only: "二零二四年" -> "2024年"
    let year_pattern = Regex::new(r"([零一二三四五六七八九〇]{4})年").unwrap();
    result = year_pattern.replace_all(&result, |caps: &regex::Captures| {
        let year_str = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let year = convert_year(year_str, &num_map);
        if year > 0 {
            format!("{}年", year)
        } else {
            caps.get(0).map(|m| m.as_str()).unwrap_or("").to_string()
        }
    }).to_string();

    // Pattern 1: Time - "三点十五分" -> "3点15分", "三点十五分三十秒" -> "3点15分30秒"
    let time_pattern = Regex::new(
        r"([零一二三四五六七八九十]+)点([零一二三四五六七八九十]+)分([零一二三四五六七八九十]+)秒"
    ).unwrap();
    result = time_pattern.replace_all(&result, |caps: &regex::Captures| {
        let hour_str = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let minute_str = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        let second_str = caps.get(3).map(|m| m.as_str()).unwrap_or("");
        
        if let (Some(hour), Some(minute), Some(second)) = (
            parse_chinese_number(hour_str, &num_map),
            parse_chinese_number(minute_str, &num_map),
            parse_chinese_number(second_str, &num_map),
        ) {
            format!("{}点{}分{}秒", hour, minute, second)
        } else {
            caps.get(0).map(|m| m.as_str()).unwrap_or("").to_string()
        }
    }).to_string();

    let time_pattern_minute = Regex::new(
        r"([零一二三四五六七八九十]+)点([零一二三四五六七八九十]+)分"
    ).unwrap();
    result = time_pattern_minute.replace_all(&result, |caps: &regex::Captures| {
        let hour_str = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let minute_str = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        
        if let (Some(hour), Some(minute)) = (
            parse_chinese_number(hour_str, &num_map),
            parse_chinese_number(minute_str, &num_map),
        ) {
            format!("{}点{}分", hour, minute)
        } else {
            caps.get(0).map(|m| m.as_str()).unwrap_or("").to_string()
        }
    }).to_string();

    // Pattern 2: Currency - "一百元" -> "100元", "三块五毛" -> "3块5毛"
    let currency_pattern = Regex::new(
        r"([零一二三四五六七八九十百千万亿]+)(元|块)([零一二三四五六七八九十]+)?(角|毛)?([零一二三四五六七八九十]+)?分?"
    ).unwrap();
    result = currency_pattern.replace_all(&result, |caps: &regex::Captures| {
        let main_str = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let unit = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        let jiao_str = caps.get(3).map(|m| m.as_str()).unwrap_or("");
        let jiao_unit = caps.get(4).map(|m| m.as_str()).unwrap_or("");
        let fen_str = caps.get(5).map(|m| m.as_str()).unwrap_or("");
        
        if let Some(main_num) = parse_chinese_number(main_str, &num_map) {
            let mut result_str = format!("{}{}", main_num, unit);
            
            if !jiao_str.is_empty() {
                if let Some(jiao_num) = parse_chinese_number(jiao_str, &num_map) {
                    result_str.push_str(&format!("{}{}", jiao_num, if jiao_unit.is_empty() { "角" } else { jiao_unit }));
                }
            }
            
            if !fen_str.is_empty() {
                if let Some(fen_num) = parse_chinese_number(fen_str, &num_map) {
                    result_str.push_str(&format!("{}分", fen_num));
                }
            }
            
            result_str
        } else {
            caps.get(0).map(|m| m.as_str()).unwrap_or("").to_string()
        }
    }).to_string();

    // Pattern 3: Percentage - "百分之五十" -> "50%"
    let percentage_pattern = Regex::new(
        r"百分之([零一二三四五六七八九十百千万亿]+)"
    ).unwrap();
    result = percentage_pattern.replace_all(&result, |caps: &regex::Captures| {
        let num_str = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        if let Some(num) = parse_chinese_number(num_str, &num_map) {
            format!("{}%", num)
        } else {
            caps.get(0).map(|m| m.as_str()).unwrap_or("").to_string()
        }
    }).to_string();

    // Pattern 4: Phone numbers - "一三八零零零零零零零零" -> "13800000000"
    // Match long sequences of digits (11+ digits for phone numbers)
    let phone_pattern = Regex::new(
        r"([零一二三四五六七八九幺]{11,})"
    ).unwrap();
    result = phone_pattern.replace_all(&result, |caps: &regex::Captures| {
        let phone_str = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let mut digits = String::new();
        for ch in phone_str.chars() {
            if let Some(&value) = num_map.get(ch.to_string().as_str()) {
                if value < 10 {
                    digits.push_str(&value.to_string());
                }
            }
        }
        if digits.len() >= 11 {
            digits
        } else {
            phone_str.to_string()
        }
    }).to_string();

    // Pattern 5: Decimals - "三点一四" -> "3.14"
    let decimal_pattern = Regex::new(
        r"([零一二三四五六七八九十百千万亿]+)点([零一二三四五六七八九零]+)"
    ).unwrap();
    result = decimal_pattern.replace_all(&result, |caps: &regex::Captures| {
        let int_str = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let dec_str = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        
        if let Some(int_part) = parse_chinese_number(int_str, &num_map) {
            let mut dec_part = String::new();
            for ch in dec_str.chars() {
                if let Some(&value) = num_map.get(ch.to_string().as_str()) {
                    if value < 10 {
                        dec_part.push_str(&value.to_string());
                    }
                }
            }
            if !dec_part.is_empty() {
                format!("{}.{}", int_part, dec_part)
            } else {
                format!("{}", int_part)
            }
        } else {
            caps.get(0).map(|m| m.as_str()).unwrap_or("").to_string()
        }
    }).to_string();

    // Pattern 6: Fractions - "三分之一" -> "1/3"
    let fraction_pattern =
        Regex::new(r"([零一二三四五六七八九十百千万亿]+)分之([零一二三四五六七八九十百千万亿]+)")
            .unwrap();
    result = fraction_pattern
        .replace_all(&result, |caps: &regex::Captures| {
            let numerator = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            let denominator = caps.get(1).map(|m| m.as_str()).unwrap_or("");

            if let (Some(num), Some(den)) = (
                parse_chinese_number(numerator, &num_map),
                parse_chinese_number(denominator, &num_map),
            ) {
                if den != 0 {
                    format!("{}/{}", num, den)
                } else {
                    caps.get(0).map(|m| m.as_str()).unwrap_or("").to_string()
                }
            } else {
                caps.get(0).map(|m| m.as_str()).unwrap_or("").to_string()
            }
        })
        .to_string();

    // Pattern 7: Range expressions - "十五六个" -> "15~16个"
    let range_pattern =
        Regex::new(r"([一二三四五六七八九十百千万亿]+)([五六七八九])(个|人|件|次|月|日|天|小时|分钟|秒)")
            .unwrap();
    result = range_pattern
        .replace_all(&result, |caps: &regex::Captures| {
            let first = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let second = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            let _unit = caps.get(3).map(|m| m.as_str()).unwrap_or("");

            if let (Some(base_num), Some(second_num)) = (
                parse_chinese_number(first, &num_map),
                parse_chinese_number(second, &num_map),
            ) {
                format!("{}~{}", base_num, base_num + second_num)
            } else {
                caps.get(0).map(|m| m.as_str()).unwrap_or("").to_string()
            }
        })
        .to_string();

    // Helper function to check if a number is part of a location name
    let is_location_context = |text: &str, match_start: usize, match_end: usize| -> bool {
        // Check if followed by a location suffix (safe slicing using char_indices)
        if match_end < text.len() {
            let text_after = text.char_indices()
                .skip_while(|(idx, _)| *idx < match_end)
                .map(|(_, ch)| ch)
                .collect::<String>();
            for suffix in LOCATION_SUFFIXES.iter() {
                if text_after.starts_with(suffix) {
                    return true;
                }
            }
        }
        
        // Check if preceded by direction words (for patterns like "西二旗")
        if match_start > 0 {
            // Convert byte indices to char indices for safe slicing
            let text_before = text.char_indices()
                .take_while(|(idx, _)| *idx < match_start)
                .map(|(_, ch)| ch)
                .collect::<String>();
            let direction_words = ["东", "西", "南", "北", "上", "下", "前", "后", "左", "右", "中"];
            for dir in direction_words.iter() {
                if text_before.ends_with(dir) && match_end < text.len() {
                    // Check if followed by a location suffix (safe slicing)
                    let text_after = text.char_indices()
                        .skip_while(|(idx, _)| *idx < match_end)
                        .map(|(_, ch)| ch)
                        .collect::<String>();
                    for suffix in LOCATION_SUFFIXES.iter() {
                        if text_after.starts_with(suffix) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    };
    
    // Pattern 8: Simple consecutive digits (一二三四五六七八 -> 12345678)
    // Match sequences of single-digit Chinese numbers without multipliers
    // BUT skip if it's part of a location name (e.g., "上地八街", "西二旗")
    let consecutive_digits_pattern = Regex::new(r"([一二三四五六七八九零〇幺]{2,})").unwrap();
    let mut new_result = String::new();
    let mut last_end = 0;
    
    for cap in consecutive_digits_pattern.find_iter(&result) {
        let match_start = cap.start();
        let match_end = cap.end();
        let chinese_num = cap.as_str();
        
        // Add text before this match (safe slicing using char_indices)
        if last_end < match_start {
            new_result.push_str(
                &result.char_indices()
                    .skip_while(|(idx, _)| *idx < last_end)
                    .take_while(|(idx, _)| *idx < match_start)
                    .map(|(_, ch)| ch)
                    .collect::<String>()
            );
        }
        
        // Check if this contains any multipliers
        let has_multipliers = chinese_num.contains("十") || chinese_num.contains("百") || chinese_num.contains("千") 
            || chinese_num.contains("万") || chinese_num.contains("亿") || chinese_num.contains("拾")
            || chinese_num.contains("佰") || chinese_num.contains("仟");
        
        // Check if it's a location name
        let is_location = is_location_context(&result, match_start, match_end);
        
        if has_multipliers || is_location {
            // Don't convert, keep original
            new_result.push_str(chinese_num);
        } else {
            // Convert each character individually
            let mut digits = String::new();
            for ch in chinese_num.chars() {
                if let Some(&value) = num_map.get(ch.to_string().as_str()) {
                    if value < 10 {
                        digits.push_str(&value.to_string());
                    } else {
                        digits.clear();
                        break;
                    }
                }
            }
            if !digits.is_empty() {
                new_result.push_str(&digits);
            } else {
                new_result.push_str(chinese_num);
            }
        }
        
        last_end = match_end;
    }
    
    // Add remaining text (safe slicing using char_indices)
    if last_end < result.len() {
        new_result.push_str(
            &result.char_indices()
                .skip_while(|(idx, _)| *idx < last_end)
                .map(|(_, ch)| ch)
                .collect::<String>()
        );
    }
    result = new_result;

    // Pattern 9: Complex numbers (十五、一百二十三 -> 15、123)
    // Match numbers that contain multipliers
    // BUT skip if it's part of a location name
    let number_pattern = Regex::new(r"([零一二三四五六七八九十百千万亿〇幺两拾佰仟萬億]+)").unwrap();
    let mut new_result = String::new();
    let mut last_end = 0;
    
    for cap in number_pattern.find_iter(&result) {
        let match_start = cap.start();
        let match_end = cap.end();
        let chinese_num = cap.as_str();
        
        // Add text before this match (safe slicing using char_indices)
        if last_end < match_start {
            new_result.push_str(
                &result.char_indices()
                    .skip_while(|(idx, _)| *idx < last_end)
                    .take_while(|(idx, _)| *idx < match_start)
                    .map(|(_, ch)| ch)
                    .collect::<String>()
            );
        }
        
        // Skip if already converted (contains Arabic digits)
        if chinese_num.chars().any(|c| c.is_ascii_digit()) {
            new_result.push_str(chinese_num);
            last_end = match_end;
            continue;
        }
        
        // Skip if it's part of a larger pattern (already processed)
        if chinese_num.contains("点") || chinese_num.contains("分之") || chinese_num.contains("%") {
            new_result.push_str(chinese_num);
            last_end = match_end;
            continue;
        }
        
        // Check if it's a location name
        let is_location = is_location_context(&result, match_start, match_end);
        
        if is_location {
            // Don't convert, keep original
            new_result.push_str(chinese_num);
        } else {
            if let Some(num) = parse_chinese_number(chinese_num, &num_map) {
                new_result.push_str(&num.to_string());
            } else {
                new_result.push_str(chinese_num);
            }
        }
        
        last_end = match_end;
    }
    
    // Add remaining text (safe slicing using char_indices)
    if last_end < result.len() {
        new_result.push_str(
            &result.char_indices()
                .skip_while(|(idx, _)| *idx < last_end)
                .map(|(_, ch)| ch)
                .collect::<String>()
        );
    }
    result = new_result;

    // Restore fixed phrases by replacing markers with original phrases
    for (marker, original_phrase) in phrase_map.iter() {
        result = result.replace(marker, original_phrase);
    }

    // #region agent log
    if original != result {
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("/Users/thinkre/Desktop/projects/KeVoiceInput/.cursor/debug.log")
        {
            let _ = std::io::Write::write_all(&mut file, format!(r#"{{"id":"log_{}","timestamp":{},"location":"text.rs:325","message":"ITN output","data":{{"original":"{}","itnResult":"{}","changed":true}},"sessionId":"debug-session","runId":"itn-debug","hypothesisId":"A"}}"#, 
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                original.replace('"', r#"\""#),
                result.replace('"', r#"\""#)
            ).as_bytes());
        }
    } else {
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("/Users/thinkre/Desktop/projects/KeVoiceInput/.cursor/debug.log")
        {
            let _ = std::io::Write::write_all(&mut file, format!(r#"{{"id":"log_{}","timestamp":{},"location":"text.rs:325","message":"ITN output","data":{{"original":"{}","itnResult":"{}","changed":false}},"sessionId":"debug-session","runId":"itn-debug","hypothesisId":"A"}}"#, 
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                original.replace('"', r#"\""#),
                result.replace('"', r#"\""#)
            ).as_bytes());
        }
    }
    // #endregion

    result
}

/// Convert year string (4 digits) to Arabic numerals
fn convert_year(year_str: &str, num_map: &HashMap<&str, i64>) -> i64 {
    let mut year = String::new();
    for ch in year_str.chars() {
        if let Some(&num) = num_map.get(ch.to_string().as_str()) {
            if num < 10 {
                year.push_str(&num.to_string());
            }
        }
    }
    year.parse().unwrap_or(0)
}

/// Parse Chinese number string to integer
/// Supports numbers like: 一、十五、一百二十三、三千四百万、一亿二千三百四十五万
/// Enhanced to support larger units (兆、京) and traditional forms
/// Algorithm: Split by large units (京、兆、亿、万), parse each segment, then combine
fn parse_chinese_number(s: &str, num_map: &HashMap<&str, i64>) -> Option<i64> {
    if s.is_empty() {
        return None;
    }

    let chars: Vec<char> = s.chars().collect();
    let mut result = 0i64;

    // Handle large units in order: 京 (10^16), 兆 (10^12), 亿 (10^8), 万 (10^4)
    // Process from largest to smallest
    
    // Check for 京 (10^16)
    if let Some(jing_idx) = chars.iter().position(|&ch| ch == '京') {
        let before_jing: String = chars[..jing_idx].iter().collect();
        let after_jing: String = chars[jing_idx + 1..].iter().collect();
        let before_value = parse_chinese_number(&before_jing, num_map).unwrap_or(0);
        result += before_value * 10000000000000000i64;
        if !after_jing.is_empty() {
            if let Some(after_value) = parse_chinese_number(&after_jing, num_map) {
                result += after_value;
            }
        }
        return if result == 0 { None } else { Some(result) };
    }

    // Check for 兆 (10^12)
    if let Some(zhao_idx) = chars.iter().position(|&ch| ch == '兆') {
        let before_zhao: String = chars[..zhao_idx].iter().collect();
        let after_zhao: String = chars[zhao_idx + 1..].iter().collect();
        let before_value = parse_chinese_number(&before_zhao, num_map).unwrap_or(0);
        result += before_value * 1000000000000i64;
        if !after_zhao.is_empty() {
            if let Some(after_value) = parse_chinese_number(&after_zhao, num_map) {
                result += after_value;
            }
        }
        return if result == 0 { None } else { Some(result) };
    }

    // Check for 亿 (10^8)
    if let Some(yi_idx) = chars.iter().position(|&ch| ch == '亿' || ch == '億') {
        let before_yi: String = chars[..yi_idx].iter().collect();
        let after_yi: String = chars[yi_idx + 1..].iter().collect();

        let before_value = parse_chinese_number(&before_yi, num_map).unwrap_or(0);
        result += before_value * 100000000;

        if !after_yi.is_empty() {
            if let Some(after_value) = parse_chinese_number(&after_yi, num_map) {
                result += after_value;
            }
        }
        return if result == 0 { None } else { Some(result) };
    }

    // Check for 万 (10^4)
    if let Some(wan_idx) = chars.iter().position(|&ch| ch == '万' || ch == '萬') {
        let before_wan: String = chars[..wan_idx].iter().collect();
        let after_wan: String = chars[wan_idx + 1..].iter().collect();

        let before_value = parse_chinese_number(&before_wan, num_map).unwrap_or(0);
        result += before_value * 10000;

        if !after_wan.is_empty() {
            if let Some(after_value) = parse_chinese_number(&after_wan, num_map) {
                result += after_value;
            }
        }
        return if result == 0 { None } else { Some(result) };
    }

    // No large units, parse the segment directly
    result = parse_segment(s, num_map);

    // #region agent log
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/Users/thinkre/Desktop/projects/KeVoiceInput/.cursor/debug.log")
    {
        let _ = std::io::Write::write_all(&mut file, format!(r#"{{"id":"log_{}","timestamp":{},"location":"text.rs:462","message":"parse_chinese_number","data":{{"input":"{}","output":{}}},"sessionId":"debug-session","runId":"itn-test","hypothesisId":"H"}}"#, 
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
            s.replace('"', r#"\""#),
            result
        ).as_bytes());
    }
    // #endregion

    if result == 0 {
        None
    } else {
        Some(result)
    }
}

/// Parse a segment without large units (万、亿、兆、京)
/// Handles 十、百、千 and their traditional forms (拾、佰、仟)
/// Process from left to right: accumulate digits, multiply by unit when encountered, then add to result
fn parse_segment(s: &str, num_map: &HashMap<&str, i64>) -> i64 {
    if s.is_empty() {
        return 0;
    }

    let chars: Vec<char> = s.chars().collect();
    let mut result = 0i64;
    let mut temp = 0i64;

    // Process from left to right
    for &ch in chars.iter() {
        let ch_str = ch.to_string();
        if let Some(&value) = num_map.get(ch_str.as_str()) {
            if value < 10 {
                // Single digit: 一、二、三... or 零、〇
                temp += value;
            } else if value == 10 {
                // 十 or 拾: if temp is 0, it means "十" (10), otherwise multiply temp by 10
                if temp == 0 {
                    temp = 10;
                } else {
                    temp *= 10;
                }
            } else if value == 100 {
                // 百 or 佰: multiply temp by 100, add to result, reset temp
                if temp == 0 {
                    temp = 1;
                }
                result += temp * 100;
                temp = 0;
            } else if value == 1000 {
                // 千 or 仟: multiply temp by 1000, add to result, reset temp
                if temp == 0 {
                    temp = 1;
                }
                result += temp * 1000;
                temp = 0;
            }
        }
    }

    // Add remaining temp (for cases like "十五" where there's no larger unit)
    result += temp;

    // #region agent log
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/Users/thinkre/Desktop/projects/KeVoiceInput/.cursor/debug.log")
    {
        let _ = std::io::Write::write_all(&mut file, format!(r#"{{"id":"log_{}","timestamp":{},"location":"text.rs:524","message":"parse_segment","data":{{"input":"{}","output":{}}},"sessionId":"debug-session","runId":"itn-test","hypothesisId":"I"}}"#, 
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
            s.replace('"', r#"\""#),
            result
        ).as_bytes());
    }
    // #endregion

    result
}

/// Hot rule types
#[derive(Debug, Clone)]
pub enum HotRule {
    Simple { from: String, to: String },
    Regex { pattern: Regex, replacement: String },
}

unsafe impl Send for HotRule {}
unsafe impl Sync for HotRule {}

/// Load hot rules from file
pub fn load_hot_rules(file_path: &PathBuf) -> Vec<HotRule> {
    let mut rules = Vec::new();

    if !file_path.exists() {
        debug!("Hot rules file does not exist: {:?}", file_path);
        return rules;
    }

    match fs::read_to_string(file_path) {
        Ok(content) => {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }

                // Check if it's a regex rule: /pattern/replacement/flags
                if line.starts_with('/') {
                    if let Some(last_slash) = line.rfind('/') {
                        if last_slash > 0 {
                            let pattern_str = &line[1..last_slash];
                            let replacement =
                                line[last_slash + 1..].trim_start_matches('/').to_string();

                            match Regex::new(pattern_str) {
                                Ok(pattern) => {
                                    rules.push(HotRule::Regex {
                                        pattern,
                                        replacement,
                                    });
                                }
                                Err(e) => {
                                    warn!(
                                        "Invalid regex pattern in hot-rule.txt: {} - {}",
                                        pattern_str, e
                                    );
                                }
                            }
                        }
                    }
                } else if line.contains('=') {
                    // Simple rule: from=to
                    let parts: Vec<&str> = line.splitn(2, '=').collect();
                    if parts.len() == 2 {
                        let from = parts[0].trim().to_string();
                        let to = parts[1].trim().to_string();
                        if !from.is_empty() {
                            rules.push(HotRule::Simple { from, to });
                        }
                    }
                }
            }
        }
        Err(e) => {
            warn!(
                "Failed to read hot rules file: {} - {}",
                file_path.display(),
                e
            );
        }
    }

    debug!("Loaded {} hot rules from {:?}", rules.len(), file_path);
    rules
}

/// Apply hot rules to text
pub fn apply_hot_rules(text: &str, rules: &[HotRule]) -> String {
    let mut result = text.to_string();

    for rule in rules {
        match rule {
            HotRule::Simple { from, to } => {
                result = result.replace(from, to);
            }
            HotRule::Regex {
                pattern,
                replacement,
            } => {
                result = pattern
                    .replace_all(&result, replacement.as_str())
                    .to_string();
            }
        }
    }

    result
}

/// Rectify record structure
#[derive(Debug, Clone)]
pub struct RectifyRecord {
    pub original: String,
    pub corrected: String,
    pub timestamp: i64,
}

/// Load rectify records from file
pub fn load_rectify_records(file_path: &PathBuf, max_records: usize) -> Vec<RectifyRecord> {
    let mut records = Vec::new();

    if !file_path.exists() {
        debug!("Rectify records file does not exist: {:?}", file_path);
        return records;
    }

    match fs::read_to_string(file_path) {
        Ok(content) => {
            for line in content.lines().rev().take(max_records) {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }

                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 2 {
                    let original = parts[0].trim().to_string();
                    let corrected = parts[1].trim().to_string();
                    let timestamp = if parts.len() >= 3 {
                        parts[2].trim().parse().unwrap_or(0)
                    } else {
                        0
                    };

                    if !original.is_empty() && !corrected.is_empty() {
                        records.push(RectifyRecord {
                            original,
                            corrected,
                            timestamp,
                        });
                    }
                }
            }
        }
        Err(e) => {
            warn!(
                "Failed to read rectify records file: {} - {}",
                file_path.display(),
                e
            );
        }
    }

    // Reverse to get chronological order
    records.reverse();
    debug!(
        "Loaded {} rectify records from {:?}",
        records.len(),
        file_path
    );
    records
}

/// Add a rectify record to file
pub fn add_rectify_record(
    file_path: &PathBuf,
    original: &str,
    corrected: &str,
) -> Result<(), String> {
    use std::fs::OpenOptions;
    use std::io::Write;

    // Create parent directory if it doesn't exist
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("Failed to get timestamp: {}", e))?
        .as_secs() as i64;

    let record_line = format!("{}|{}|{}\n", original, corrected, timestamp);

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)
        .map_err(|e| format!("Failed to open rectify file: {}", e))?;

    file.write_all(record_line.as_bytes())
        .map_err(|e| format!("Failed to write rectify record: {}", e))?;

    debug!("Added rectify record: {} -> {}", original, corrected);
    Ok(())
}

/// Format rectify records as context for LLM
pub fn format_rectify_context(records: &[RectifyRecord]) -> String {
    if records.is_empty() {
        return String::new();
    }

    let mut context = String::from("以下是之前的纠错记录，请参考这些例子来纠正文本：\n");
    for record in records.iter().take(10) {
        context.push_str(&format!(
            "原文：{}\n纠正：{}\n\n",
            record.original, record.corrected
        ));
    }

    context
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_custom_words_exact_match() {
        let text = "hello world";
        let custom_words = vec!["Hello".to_string(), "World".to_string()];
        let result = apply_custom_words(text, &custom_words, 0.5);
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_apply_custom_words_fuzzy_match() {
        let text = "helo wrold";
        let custom_words = vec!["hello".to_string(), "world".to_string()];
        let result = apply_custom_words(text, &custom_words, 0.5);
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_preserve_case_pattern() {
        assert_eq!(preserve_case_pattern("HELLO", "world"), "WORLD");
        assert_eq!(preserve_case_pattern("Hello", "world"), "World");
        assert_eq!(preserve_case_pattern("hello", "WORLD"), "WORLD");
    }

    #[test]
    fn test_extract_punctuation() {
        assert_eq!(extract_punctuation("hello"), ("", ""));
        assert_eq!(extract_punctuation("!hello?"), ("!", "?"));
        assert_eq!(extract_punctuation("...hello..."), ("...", "..."));
    }

    #[test]
    fn test_empty_custom_words() {
        let text = "hello world";
        let custom_words = vec![];
        let result = apply_custom_words(text, &custom_words, 0.5);
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_filter_filler_words() {
        let text = "So um I was thinking uh about this";
        let result = filter_transcription_output(text);
        assert_eq!(result, "So I was thinking about this");
    }

    #[test]
    fn test_filter_filler_words_case_insensitive() {
        let text = "UM this is UH a test";
        let result = filter_transcription_output(text);
        assert_eq!(result, "this is a test");
    }

    #[test]
    fn test_filter_filler_words_with_punctuation() {
        let text = "Well, um, I think, uh. that's right";
        let result = filter_transcription_output(text);
        assert_eq!(result, "Well, I think, that's right");
    }

    #[test]
    fn test_filter_cleans_whitespace() {
        let text = "Hello    world   test";
        let result = filter_transcription_output(text);
        assert_eq!(result, "Hello world test");
    }

    #[test]
    fn test_filter_trims() {
        let text = "  Hello world  ";
        let result = filter_transcription_output(text);
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_filter_combined() {
        let text = "  Um, so I was, uh, thinking about this  ";
        let result = filter_transcription_output(text);
        assert_eq!(result, "so I was, thinking about this");
    }

    #[test]
    fn test_filter_preserves_valid_text() {
        let text = "This is a completely normal sentence.";
        let result = filter_transcription_output(text);
        assert_eq!(result, "This is a completely normal sentence.");
    }

    #[test]
    fn test_filter_stutter_collapse() {
        let text = "w wh wh wh wh wh wh wh wh wh why";
        let result = filter_transcription_output(text);
        assert_eq!(result, "w wh why");
    }

    #[test]
    fn test_filter_stutter_short_words() {
        let text = "I I I I think so so so so";
        let result = filter_transcription_output(text);
        assert_eq!(result, "I think so");
    }

    #[test]
    fn test_filter_stutter_mixed_case() {
        let text = "No NO no NO no";
        let result = filter_transcription_output(text);
        assert_eq!(result, "No");
    }

    #[test]
    fn test_filter_stutter_preserves_two_repetitions() {
        let text = "no no is fine";
        let result = filter_transcription_output(text);
        assert_eq!(result, "no no is fine");
    }
}
