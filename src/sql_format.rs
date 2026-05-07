//! SQL formatting and syntax highlighting for the query detail panels.

use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

// ── Keyword sets ─────────────────────────────────────────────────────────────

/// Clause keywords that get their own line (longest first — order matters).
static CLAUSE_BREAKS: &[&str] = &[
    "LEFT OUTER JOIN",
    "RIGHT OUTER JOIN",
    "FULL OUTER JOIN",
    "INSERT INTO",
    "DELETE FROM",
    "UNION ALL",
    "GROUP BY",
    "ORDER BY",
    "INNER JOIN",
    "CROSS JOIN",
    "LEFT JOIN",
    "RIGHT JOIN",
    "FULL JOIN",
    "RETURNING",
    "INTERSECT",
    "EXCEPT",
    "SELECT",
    "OFFSET",
    "HAVING",
    "VALUES",
    "UPDATE",
    "UNION",
    "WHERE",
    "LIMIT",
    "FROM",
    "WITH",
    "JOIN",
    "SET",
    "ON",
];

static KEYWORDS: &[&str] = &[
    "SELECT", "DISTINCT", "FROM", "WHERE", "AND", "OR", "NOT",
    "IN", "EXISTS", "BETWEEN", "LIKE", "ILIKE",
    "IS", "NULL", "TRUE", "FALSE", "AS", "ON",
    "JOIN", "LEFT", "RIGHT", "INNER", "OUTER", "FULL", "CROSS",
    "UNION", "ALL", "INTERSECT", "EXCEPT", "WITH",
    "HAVING", "GROUP", "ORDER", "BY", "ASC", "DESC",
    "NULLS", "FIRST", "LAST", "LIMIT", "OFFSET",
    "INSERT", "INTO", "VALUES", "UPDATE", "SET", "DELETE", "RETURNING",
    "CASE", "WHEN", "THEN", "ELSE", "END",
    "CAST", "OVER", "PARTITION", "ROWS", "RANGE",
    "UNBOUNDED", "PRECEDING", "FOLLOWING", "CURRENT", "ROW",
    "CREATE", "DROP", "ALTER", "TABLE", "INDEX", "VIEW",
    "EXPLAIN", "ANALYZE", "VERBOSE",
    "BEGIN", "COMMIT", "ROLLBACK",
    "FILTER", "LATERAL", "RECURSIVE",
    "DEFAULT", "PRIMARY", "KEY", "REFERENCES", "UNIQUE",
];

static FUNCTIONS: &[&str] = &[
    "COUNT", "SUM", "AVG", "MIN", "MAX",
    "COALESCE", "NULLIF", "GREATEST", "LEAST",
    "NOW", "CURRENT_TIMESTAMP", "CURRENT_DATE", "CURRENT_TIME",
    "DATE_PART", "DATE_TRUNC", "EXTRACT", "AGE",
    "CONCAT", "LENGTH", "UPPER", "LOWER", "TRIM", "LTRIM", "RTRIM",
    "REPLACE", "SUBSTRING", "SPLIT_PART", "REGEXP_REPLACE",
    "TO_CHAR", "TO_DATE", "TO_TIMESTAMP", "TO_NUMBER",
    "ROUND", "FLOOR", "CEIL", "CEILING", "ABS", "MOD", "POWER", "SQRT",
    "ROW_NUMBER", "RANK", "DENSE_RANK", "PERCENT_RANK", "NTILE",
    "LAG", "LEAD", "FIRST_VALUE", "LAST_VALUE",
    "ARRAY_AGG", "STRING_AGG", "JSON_AGG", "JSONB_AGG",
    "UNNEST", "GENERATE_SERIES",
    "PG_SLEEP", "PG_CANCEL_BACKEND", "PG_TERMINATE_BACKEND",
    "PG_SIZE_PRETTY", "PG_DATABASE_SIZE", "PG_RELATION_SIZE",
    "NULLIF", "COALESCE",
];

// ── Formatter ─────────────────────────────────────────────────────────────────

/// Normalise whitespace and insert newlines before top-level SQL clauses.
pub fn format_sql(raw: &str) -> String {
    // Collapse all whitespace runs to a single space.
    let norm: String = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    let chars: Vec<char> = norm.chars().collect();
    let upper: Vec<char> = norm.to_uppercase().chars().collect();
    let n = chars.len();

    let mut out = String::with_capacity(n + 128);
    let mut in_str = false;
    let mut depth = 0i32;
    let mut i = 0;

    while i < n {
        let c = chars[i];

        // ── String literal ────────────────────────────────────────────────
        if !in_str && c == '\'' {
            in_str = true;
            out.push(c);
            i += 1;
            continue;
        }
        if in_str {
            out.push(c);
            if c == '\'' {
                if i + 1 < n && chars[i + 1] == '\'' {
                    // escaped ''
                    out.push('\'');
                    i += 2;
                } else {
                    in_str = false;
                    i += 1;
                }
            } else {
                i += 1;
            }
            continue;
        }

        // ── Parentheses ───────────────────────────────────────────────────
        if c == '(' { depth += 1; out.push(c); i += 1; continue; }
        if c == ')' { depth -= 1; out.push(c); i += 1; continue; }

        // ── Space at top level: look ahead for a clause keyword ───────────
        if c == ' ' && depth == 0 {
            let next = i + 1;
            if let Some(_kw_len) = clause_at(&upper, next) {
                // Emit newline instead of space before the clause keyword
                if !out.is_empty() { out.push('\n'); }
            } else {
                out.push(' ');
            }
            i += 1;
            continue;
        }

        out.push(c);
        i += 1;
    }

    out
}

/// Check if `upper[pos..]` starts with a clause keyword (word-boundary aware).
/// Returns the keyword length if matched.
fn clause_at(upper: &[char], pos: usize) -> Option<usize> {
    for kw in CLAUSE_BREAKS {
        let kw_chars: Vec<char> = kw.chars().collect();
        let kl = kw_chars.len();
        if pos + kl > upper.len() { continue; }
        if upper[pos..pos + kl] != kw_chars[..] { continue; }
        // Word boundary: what comes after must not be alphanumeric / underscore
        let after = pos + kl;
        let boundary = after >= upper.len()
            || (!upper[after].is_alphanumeric() && upper[after] != '_');
        if boundary { return Some(kl); }
    }
    None
}

// ── Highlighter ───────────────────────────────────────────────────────────────

/// Format + syntax-highlight SQL, returning ratatui [`Line`]s ready to render.
pub fn highlight(sql: &str) -> Vec<Line<'static>> {
    format_sql(sql)
        .lines()
        .map(highlight_line)
        .collect()
}

fn highlight_line(line: &str) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    let bytes = line.as_bytes();
    let n = bytes.len();
    let mut i = 0;

    while i < n {
        // ── Whitespace ────────────────────────────────────────────────────
        if bytes[i] == b' ' || bytes[i] == b'\t' {
            let start = i;
            while i < n && (bytes[i] == b' ' || bytes[i] == b'\t') {
                i += 1;
            }
            spans.push(Span::raw(line[start..i].to_string()));
            continue;
        }

        // ── Line comment ──────────────────────────────────────────────────
        if i + 1 < n && bytes[i] == b'-' && bytes[i + 1] == b'-' {
            spans.push(Span::styled(
                line[i..].to_string(),
                Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
            ));
            break;
        }

        // ── String literal ────────────────────────────────────────────────
        if bytes[i] == b'\'' {
            let start = i;
            i += 1;
            while i < n {
                if bytes[i] == b'\'' {
                    i += 1;
                    if i < n && bytes[i] == b'\'' { i += 1; } else { break; }
                } else {
                    i += 1;
                }
            }
            spans.push(Span::styled(
                line[start..i].to_string(),
                Style::default().fg(Color::Green),
            ));
            continue;
        }

        // ── Query parameter ($1, $2 …) ────────────────────────────────────
        if bytes[i] == b'$' && i + 1 < n && bytes[i + 1].is_ascii_digit() {
            let start = i;
            i += 1;
            while i < n && bytes[i].is_ascii_digit() { i += 1; }
            spans.push(Span::styled(
                line[start..i].to_string(),
                Style::default().fg(Color::Magenta),
            ));
            continue;
        }

        // ── Number ────────────────────────────────────────────────────────
        if bytes[i].is_ascii_digit() {
            let start = i;
            while i < n && (bytes[i].is_ascii_digit() || bytes[i] == b'.') { i += 1; }
            spans.push(Span::styled(
                line[start..i].to_string(),
                Style::default().fg(Color::Magenta),
            ));
            continue;
        }

        // ── Quoted identifier ("foo") ─────────────────────────────────────
        if bytes[i] == b'"' {
            let start = i;
            i += 1;
            while i < n && bytes[i] != b'"' { i += 1; }
            if i < n { i += 1; }
            spans.push(Span::styled(
                line[start..i].to_string(),
                Style::default().fg(Color::White),
            ));
            continue;
        }

        // ── Word (keyword, function, or identifier) ───────────────────────
        if bytes[i].is_ascii_alphabetic() || bytes[i] == b'_' {
            let start = i;
            while i < n && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            let word = &line[start..i];
            let upper = word.to_ascii_uppercase();
            let style = if KEYWORDS.iter().any(|&k| k == upper) {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else if FUNCTIONS.iter().any(|&f| f == upper) {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::White)
            };
            spans.push(Span::styled(word.to_string(), style));
            continue;
        }

        // ── Operators ─────────────────────────────────────────────────────
        if matches!(bytes[i], b'=' | b'<' | b'>' | b'!' | b'+' | b'*' | b'/' | b'%' | b'|' | b'&') {
            let start = i;
            while i < n && matches!(bytes[i], b'=' | b'<' | b'>' | b'!' | b'+' | b'*' | b'/' | b'%' | b'|' | b'&') {
                i += 1;
            }
            spans.push(Span::styled(
                line[start..i].to_string(),
                Style::default().fg(Color::Yellow),
            ));
            continue;
        }

        // ── Minus (operator or negative sign) ────────────────────────────
        if bytes[i] == b'-' {
            spans.push(Span::styled("-".to_string(), Style::default().fg(Color::Yellow)));
            i += 1;
            continue;
        }

        // ── Punctuation ───────────────────────────────────────────────────
        let punct_style = match bytes[i] {
            b'(' | b')' | b'[' | b']' => Style::default().fg(Color::DarkGray),
            b',' | b';' | b'.' | b':' => Style::default().fg(Color::DarkGray),
            _ => Style::default(),
        };
        spans.push(Span::styled(
            (bytes[i] as char).to_string(),
            punct_style,
        ));
        i += 1;
    }

    Line::from(spans)
}
