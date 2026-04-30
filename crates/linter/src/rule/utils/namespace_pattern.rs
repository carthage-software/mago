use mago_atom::starts_with_ignore_case;

const SEPARATOR: char = '\\';

pub fn matches_namespace_pattern(fqcn: &str, pattern: &str, is_constant: bool, treat_as_namespace: bool) -> bool {
    if pattern.contains('{') {
        return expand_braces(pattern)
            .iter()
            .any(|p| matches_namespace_pattern(fqcn, p, is_constant, treat_as_namespace));
    }

    if !pattern.contains('*') {
        let p = pattern.trim_matches(SEPARATOR);
        let f = fqcn.trim_matches(SEPARATOR);

        if p.is_empty() {
            return f.is_empty();
        }

        if treat_as_namespace || pattern.ends_with(SEPARATOR) {
            if !starts_with_ignore_case(f, p) {
                return false;
            }

            return f.len() == p.len() || f.as_bytes().get(p.len()) == Some(&(SEPARATOR as u8));
        }

        if is_constant {
            let f_last = f.rsplit_once(SEPARATOR);
            let p_last = p.rsplit_once(SEPARATOR);

            return match (f_last, p_last) {
                (Some((f_ns, f_name)), Some((p_ns, p_name))) => f_ns.eq_ignore_ascii_case(p_ns) && f_name == p_name,
                (None, None) => f == p,
                _ => false,
            };
        }
        return f.eq_ignore_ascii_case(p);
    }

    let fqcn = fqcn.trim_matches(SEPARATOR);
    let pattern = pattern.trim_matches(SEPARATOR);

    if pattern == "**" {
        return true;
    }

    if pattern == "*" {
        return !fqcn.contains(SEPARATOR);
    }

    if fqcn.is_empty() || pattern.is_empty() {
        return fqcn == pattern;
    }

    let fqcn_parts: Vec<&str> = fqcn.split(SEPARATOR).collect();
    let pattern_parts: Vec<&str> = pattern.split(SEPARATOR).collect();

    do_match(&fqcn_parts, &pattern_parts, is_constant)
}

fn do_match(fqcn_parts: &[&str], pattern_parts: &[&str], is_constant: bool) -> bool {
    match (fqcn_parts.first(), pattern_parts.first()) {
        (None, None) => true,
        (_, Some(&"**")) => {
            if pattern_parts.len() == 1 {
                return true;
            }
            if fqcn_parts.is_empty() {
                return do_match(fqcn_parts, &pattern_parts[1..], is_constant);
            }
            do_match(fqcn_parts, &pattern_parts[1..], is_constant)
                || do_match(&fqcn_parts[1..], pattern_parts, is_constant)
        }
        (Some(f_part), Some(p_part)) => {
            let is_last = fqcn_parts.len() == 1 && pattern_parts.len() == 1;
            let case_sensitive = is_constant && is_last;

            if segment_matches(f_part, p_part, case_sensitive) {
                do_match(&fqcn_parts[1..], &pattern_parts[1..], is_constant)
            } else {
                false
            }
        }
        (None, Some(_)) => pattern_parts.len() == 1 && pattern_parts[0] == "**",
        _ => false,
    }
}

fn segment_matches(fqcn_part: &str, pattern_part: &str, case_sensitive: bool) -> bool {
    if pattern_part == "*" {
        return true;
    }
    if !pattern_part.contains('*') {
        return if case_sensitive { fqcn_part == pattern_part } else { fqcn_part.eq_ignore_ascii_case(pattern_part) };
    }

    let p_chunks: Vec<&str> = pattern_part.split('*').collect();
    let mut remainder = fqcn_part;

    if !p_chunks[0].is_empty() {
        if remainder.len() < p_chunks[0].len() {
            return false;
        }
        let prefix = &remainder[..p_chunks[0].len()];
        if !(if case_sensitive { prefix == p_chunks[0] } else { prefix.eq_ignore_ascii_case(p_chunks[0]) }) {
            return false;
        }
        remainder = &remainder[p_chunks[0].len()..];
    }

    let last_chunk = p_chunks.last().unwrap();
    if !last_chunk.is_empty() {
        if remainder.len() < last_chunk.len() {
            return false;
        }
        let suffix = &remainder[remainder.len() - last_chunk.len()..];
        if !(if case_sensitive { suffix == *last_chunk } else { suffix.eq_ignore_ascii_case(last_chunk) }) {
            return false;
        }
        remainder = &remainder[..remainder.len() - last_chunk.len()];
    }

    for chunk in &p_chunks[1..p_chunks.len() - 1] {
        if chunk.is_empty() {
            continue;
        }
        let found = if case_sensitive { remainder.find(chunk) } else { find_ignore_ascii_case(remainder, chunk) };
        if let Some(pos) = found {
            remainder = &remainder[pos + chunk.len()..];
        } else {
            return false;
        }
    }

    true
}

fn find_ignore_ascii_case(haystack: &str, needle: &str) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    haystack.as_bytes().windows(needle.len()).position(|window| window.eq_ignore_ascii_case(needle.as_bytes()))
}

fn expand_braces(pattern: &str) -> Vec<String> {
    let Some(open) = pattern.find('{') else {
        return vec![pattern.to_string()];
    };

    let mut depth = 0;
    let mut close = None;
    for (i, ch) in pattern[open..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    close = Some(open + i);
                    break;
                }
            }
            _ => {}
        }
    }

    let Some(close) = close else {
        return vec![pattern.to_string()];
    };

    let prefix = &pattern[..open];
    let suffix = &pattern[close + 1..];
    let alternatives = &pattern[open + 1..close];

    let mut parts = Vec::new();
    let mut depth = 0;
    let mut start = 0;
    for (i, ch) in alternatives.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(&alternatives[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }

    parts.push(&alternatives[start..]);

    let mut results = Vec::with_capacity(parts.len());
    for part in parts {
        let expanded = format!("{prefix}{part}{suffix}");
        results.extend(expand_braces(&expanded));
    }

    results
}
