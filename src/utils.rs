pub fn format_duration(duration_seconds: u64) -> String {
    let seconds = duration_seconds % 60;
    let minutes = (duration_seconds / 60) % 60;
    let hours = duration_seconds / 3600;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

pub fn parse_time_string_to_seconds(input: &str) -> Result<u64, String> {
    let has_hours = input.contains('h');
    let has_minutes = input.contains('m');
    if !has_hours && !has_minutes {
        return Err("error parsing time".into());
    }
    let mut hours: u64 = 0;
    let mut minutes: u64 = 0;
    if has_hours {
        let h_index = input.find('h').unwrap();
        let split_at = input.split_at(h_index);
        hours += String::from(split_at.0)
            .parse::<u64>()
            .expect("could not extract hours");
    }
    if has_minutes {
        let m_index = input.find('m').unwrap();
        let h_index = match input.find('h') {
            Some(index) => index + 1,
            None => 0,
        };
        let slice = &input[h_index..m_index];
        minutes += String::from(slice)
            .parse::<u64>()
            .expect("could not extract hours");
    }
    Ok(hours * 3600 + minutes * 60)
}
