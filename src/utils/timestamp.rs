pub fn current_timestamp() -> u64 {
    Utc::now().timestamp_millis() as u64
}


pub fn months_to_milliseconds(months: i64) -> i64 {
    let current_time = Utc::now();
    let target_time = current_time
        .checked_add_signed(Duration::days(months * 30))
        .unwrap();
    let target_naive = target_time.naive_utc();
    let difference = target_naive.timestamp_millis() - current_time.timestamp_millis();

    difference
}
