use sanitise::sanitise;

fn main() {
    let names = vec![
        Some("John".to_string()),
        Some("Jane".to_string()),
        Some("Jack".to_string()),
    ];
    let ages: Vec<Option<i64>> = vec![Some(17), Some(26), Some(103)];
    let ratings = vec![
        Some("5".to_string()),
        Some("1".to_string()),
        Some("10".to_string()),
    ];

    let (_, (names, ages, ratings)) =
        sanitise!(include_str!("sanitise.yaml"), (&names, &ages, &ratings)).unwrap();
    dbg!(names, ages, ratings);
}
