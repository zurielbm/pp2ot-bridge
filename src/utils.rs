/// Parse a duration string (HH:MM:SS) to milliseconds
pub fn parse_duration_to_ms(duration: &str) -> u64 {
    let parts: Vec<&str> = duration.split(':').collect();
    if parts.len() == 3 {
        let hours: u64 = parts[0].parse().unwrap_or(0);
        let minutes: u64 = parts[1].parse().unwrap_or(0);
        let seconds: u64 = parts[2].parse().unwrap_or(0);
        (hours * 3600 + minutes * 60 + seconds) * 1000
    } else {
        300000 // Default 5 minutes
    }
}

/// Format milliseconds to duration string (HH:MM:SS)
pub fn format_ms_to_duration(ms: u64) -> String {
    let seconds = ms / 1000;
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, secs)
}
