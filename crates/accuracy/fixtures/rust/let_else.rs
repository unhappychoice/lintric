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
