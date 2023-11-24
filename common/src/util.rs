pub fn seconds_to_time_string(mut seconds: i32) -> String {
    let mut minutes = 0;
    while seconds >= 60 {
        minutes += 1;
        seconds -= 60;
    }
    format!("{:02}:{:02}", minutes, seconds)
}

pub fn m_to_n_mi(m: f32) -> f32 {
    m / 1852.0
}

pub fn knots_to_m_per_s(knots: f32) -> f32 {
    knots * 0.51444
}