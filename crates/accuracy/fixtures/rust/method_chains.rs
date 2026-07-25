// Iterator chains and closures passed as arguments.

struct Item {
    weight: i32,
}

fn total(items: &[Item]) -> i32 { //~ depends: Item@3
    items //~ depends: items@7
        .iter()
        .map(|item| item.weight) //~ depends: weight@4
        .filter(|weight| *weight > 0)
        .sum()
}

fn main() {
    let items = vec![Item { weight: 1 }]; //~ depends: Item@3, weight@4
    let heaviest = items.iter().map(|i| i.weight).max(); //~ depends: items@16, weight@4
    let _ = (total(&items), heaviest); //~ depends: total@7, items@16, heaviest@17
}
