use sanitise::sanitise;

fn main() {
    for ((time_millis, pulse, movement), (time_secs,)) in sanitise!(
        include_str!("sanity.yaml"),
        "time,pulse,movement\n5,60,1\n23,73,0".to_string()
    )
    .unwrap()
    {
        println!(
            "time_millis: {:#?}, pulse: {:#?}, movement: {:#?}, time_secs: {:#?}",
            time_millis, pulse, movement, time_secs
        );
    }
}
