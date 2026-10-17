use regex::Regex;
use std::fmt;

#[derive(Debug, Clone)]
pub struct SubtitleEntry {
    pub index: usize,
    pub start_time: String,
    pub end_time: String,
    pub text: String,
}

impl fmt::Display for SubtitleEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}\n{} --> {}\n{}\n",
            self.index, self.start_time, self.end_time, self.text
        )
    }
}

pub fn parse_srt(content: &str) -> Vec<SubtitleEntry> {
    let mut entries = Vec::new();
    let normalized = content.replace("\r\n", "\n").replace('\r', "\n");
    let blocks = normalized.split("\n\n");

    let timestamp_regex = Regex::new(r"(\d{2}:\d{2}:\d{2}[,\.]\d{3})\s*-->\s*(\d{2}:\d{2}:\d{2}[,\.]\d{3})").unwrap();

    for block in blocks {
        let lines: Vec<&str> = block.trim().lines().collect();
        if lines.is_empty() {
            continue;
        }

        // Try to find timestamp line
        let mut ts_idx = None;
        let mut start_ts = String::new();
        let mut end_ts = String::new();

        for (idx, line) in lines.iter().enumerate() {
            if let Some(caps) = timestamp_regex.captures(line) {
                ts_idx = Some(idx);
                start_ts = caps.get(1).unwrap().as_str().replace('.', ",");
                end_ts = caps.get(2).unwrap().as_str().replace('.', ",");
                break;
            }
        }

        if let Some(ts_idx) = ts_idx {
            let index = if ts_idx > 0 {
                lines[0].trim().parse::<usize>().unwrap_or(entries.len() + 1)
            } else {
                entries.len() + 1
            };

            let text_lines: Vec<&str> = lines[(ts_idx + 1)..].iter().copied().collect();
            let text = text_lines.join("\n");

            entries.push(SubtitleEntry {
                index,
                start_time: start_ts,
                end_time: end_ts,
                text,
            });
        }
    }

    entries
}

pub fn serialize_srt(entries: &[SubtitleEntry]) -> String {
    let mut out = String::new();
    for (i, entry) in entries.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str(&format!(
            "{}\n{} --> {}\n{}\n",
            entry.index, entry.start_time, entry.end_time, entry.text
        ));
    }
    out
}

pub fn is_untranslatable(text: &str) -> bool {
    let cleaned = text.trim();
    if cleaned.is_empty() {
        return true;
    }

    // Only numbers and basic punctuation
    let punc_or_num = cleaned.chars().all(|c| {
        c.is_numeric()
            || c.is_whitespace()
            || matches!(
                c,
                '.' | ',' | ';' | ':' | '!' | '?' | '-' | '—' | '–' | '(' | ')' | '[' | ']' | '"' | '\'' | '/' | '\\'
            )
    });
    if punc_or_num {
        return true;
    }

    // Sound tags like [music], (applause), ♪ music ♪, etc.
    if (cleaned.starts_with('[') && cleaned.ends_with(']'))
        || (cleaned.starts_with('(') && cleaned.ends_with(')'))
        || (cleaned.starts_with('{') && cleaned.ends_with('}'))
        || (cleaned.starts_with('<') && cleaned.ends_with('>'))
        || (cleaned.starts_with('♪') && cleaned.ends_with('♪'))
        || (cleaned.starts_with('♫') && cleaned.ends_with('♫'))
    {
        return true;
    }

    false
}
