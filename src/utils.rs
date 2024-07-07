pub fn format_duration(duration_seconds: u64) -> String {
    let seconds = duration_seconds % 60;
    let minutes = (duration_seconds / 60) % 60;
    let hours = duration_seconds / 3600;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}
