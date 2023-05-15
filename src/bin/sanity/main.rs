use asylum::sanitise;

fn main() {
    println!("{}", sanitise!(include_str!("sanity.yaml"), ""));
}
