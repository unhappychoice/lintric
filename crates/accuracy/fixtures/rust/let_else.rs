// `let else`, `if let` and `while let` bindings.

enum Slot {
    Filled(i32),
    Empty,
}

fn read(slot: Slot) -> i32 { //~ depends: Slot@3
    let Slot::Filled(value) = slot else { //~ depends: Slot@3, Filled@4, slot@8
        return 0;
    };
    value //~ depends: value@9
}

fn main() {
    let slot = Slot::Filled(1); //~ depends: Slot@3, Filled@4
    if let Slot::Empty = slot { //~ depends: Slot@3, Empty@5, slot@16
        return;
    }
    let _ = read(slot); //~ depends: read@8, slot@16
}

// A `let` chain holds several conditions, so each binds its own names and the body reads all of them.
fn both(first: Slot, second: Slot) -> i32 { //~ depends: Slot@3
    if let Slot::Filled(a) = first //~ depends: Slot@3, Filled@4, first@24
        && let Slot::Filled(b) = second //~ depends: Slot@3, Filled@4, second@24
    {
        a + b //~ depends: a@25, b@26
    } else {
        0
    }
}

fn until_empty(mut slot: Slot) -> i32 { //~ depends: Slot@3
    let mut total = 0;
    while let Slot::Filled(step) = slot //~ depends: Slot@3, Filled@4, slot@34
        && step > 0 //~ depends: step@36
    {
        total += step; //~ depends: total@35, step@36
        slot = Slot::Empty; //~ depends: slot@34, Slot@3, Empty@5
    }
    total //~ depends: total@35
}
