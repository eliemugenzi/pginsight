#![allow(dead_code)]

use chrono::{DateTime, Utc};
use std::time::Duration;

/// Format a byte count for human consumption (e.g. "1.2 MiB").
pub fn bytes(n: i64) -> String {
    const UNITS: [&str; 6] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];
    if n < 0 {
        return "-".into();
    }
    let mut value = n as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{} {}", n, UNITS[unit])
    } else {
        format!("{:.1} {}", value, UNITS[unit])
    }
}

/// Format a duration like `2m 13s` or `843ms`.
pub fn duration(d: Duration) -> String {
    let total_ms = d.as_millis();
    if total_ms < 1000 {
        return format!("{}ms", total_ms);
    }
    let secs = d.as_secs();
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    if h > 0 {
        format!("{}h {:02}m", h, m)
    } else if m > 0 {
        format!("{}m {:02}s", m, s)
    } else {
        format!("{}s", s)
    }
}

/// Format a fraction in [0,1] as a percent.
pub fn pct(frac: f64) -> String {
    if !frac.is_finite() {
        return "-".into();
    }
    format!("{:.1}%", frac * 100.0)
}

/// Format milliseconds into a human-readable duration.
pub fn ms(millis: f64) -> String {
    if millis < 1.0 {
        format!("{:.2}ms", millis)
    } else if millis < 1_000.0 {
        format!("{:.0}ms", millis)
    } else if millis < 60_000.0 {
        format!("{:.1}s", millis / 1_000.0)
    } else if millis < 3_600_000.0 {
        let m = millis as u64 / 60_000;
        let s = (millis as u64 % 60_000) / 1_000;
        format!("{}m {:02}s", m, s)
    } else {
        let h = millis as u64 / 3_600_000;
        let m = (millis as u64 % 3_600_000) / 60_000;
        format!("{}h {:02}m", h, m)
    }
}

/// Format an integer with thousands separators.
pub fn number(n: i64) -> String {
    if n < 0 {
        return format!("-{}", number(-n));
    }
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

/// Format a UTC timestamp.
pub fn datetime(dt: &DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}
