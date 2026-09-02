//! # `urae_notebook::notebook::line_shift`
//!
//! Dynamic Line Index Shifting and Smart Variable Refactoring.

/// Reconcile and rewrite line reference numbers ($N, $M..$N) when lines are inserted or deleted.
pub fn reconcile_line_references_on_shift(
    old_lines: &[&str],
    new_lines: &[&str],
) -> Option<String> {
    if old_lines.len() == new_lines.len() {
        return None;
    }

    let mut prefix_match = 0;
    while prefix_match < old_lines.len()
        && prefix_match < new_lines.len()
        && old_lines[prefix_match] == new_lines[prefix_match]
    {
        prefix_match += 1;
    }

    let delta: i64 = (new_lines.len() as i64) - (old_lines.len() as i64);
    if delta == 0 {
        return None;
    }

    let shift_point = prefix_match + 1;

    let mut rewritten_lines = Vec::with_capacity(new_lines.len());
    let mut modified = false;

    for (idx, &line) in new_lines.iter().enumerate() {
        let cur_line_num = idx + 1;
        if cur_line_num <= shift_point || !line.contains('$') {
            rewritten_lines.push(line.to_string());
            continue;
        }

        let mut new_line = String::new();
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            if chars[i] == '$' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit() {
                let mut j = i + 1;
                while j < chars.len() && chars[j].is_ascii_digit() {
                    j += 1;
                }
                let num_str: String = chars[i + 1..j].iter().collect();
                if let Ok(line_num) = num_str.parse::<usize>() {
                    if line_num >= shift_point {
                        let shifted = (line_num as i64 + delta).max(1) as usize;
                        new_line.push_str(&format!("${}", shifted));
                        modified = true;
                        i = j;
                        continue;
                    }
                }
            }
            new_line.push(chars[i]);
            i += 1;
        }
        rewritten_lines.push(new_line);
    }

    if modified {
        Some(rewritten_lines.join("\n"))
    } else {
        None
    }
}
