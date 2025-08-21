fn main() {
    let option_value = Some(10);
    if let Some(value) = option_value {
        println!("Value: {}", value);
    }

    let numbers = vec![1, 2, 3];
    for num in numbers {
        println!("Number: {}", num);
    }

    let mut iter = Some(5);
    while let Some(i) = iter {
        println!("Iteration: {}", i);
        iter = if i > 0 { Some(i - 1) } else { None };
    }
}
