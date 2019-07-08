use interface::plugin;

plugin! { name: "mul"; main: mul }

fn mul(data: Vec<u64>) -> u64 {
    data.into_iter().fold(0, |acc, x| acc * x)
}
