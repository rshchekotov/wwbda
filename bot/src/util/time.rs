pub fn format_duration(total_secs: u64) -> String {
    let mut secs = total_secs;

    let days = secs / 86_400;
    secs %= 86_400;

    let hours = secs / 3_600;
    secs %= 3_600;

    let minutes = secs / 60;
    secs %= 60;

    let mut parts = Vec::new();
    if days != 0 {
        parts.push(format!("{d}d", d = days));
    }
    if hours != 0 {
        parts.push(format!("{h}h", h = hours));
    }
    if minutes != 0 {
        parts.push(format!("{m}m", m = minutes));
    }
    if secs != 0 || parts.is_empty() {
        parts.push(format!("{s}s", s = secs));
    }

    parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_formatting() {
        assert_eq!("14d", format_duration(1_209_600));
        assert_eq!("1m", format_duration(60));
    }
}
