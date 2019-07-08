use interface::plugin;

plugin! { name: "sum"; main: sum }

fn sum(data: Vec<u64>) -> u64 {
    data.into_iter().fold(0, |acc, x| acc + x)
}
