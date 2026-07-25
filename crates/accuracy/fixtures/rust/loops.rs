// Loop constructs and the bindings they introduce.

fn main() {
    let items = vec![1, 2, 3];
    let mut total = 0;

    for item in &items { //~ depends: items@4
        total += item; //~ depends: total@5, item@7
    }

    let mut countdown = 3;
    while countdown > 0 { //~ depends: countdown@11
        countdown -= 1; //~ depends: countdown@11
    }

    loop {
        if total > 0 { //~ depends: total@5
            break;
        }
    }

    let _ = total; //~ depends: total@5
}
