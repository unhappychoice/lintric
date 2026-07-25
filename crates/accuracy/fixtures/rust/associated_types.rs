// Associated types in traits and their implementations.
//
// `Self::Item` inside the impl names the impl's own alias, which is nearer than the trait's
// declaration. `Self` itself is the type inside an impl and the trait inside a trait.

trait Container {
    type Item;

    fn first(&self) -> Self::Item; //~ depends: Container@6, Item@7
}

struct Numbers;

impl Container for Numbers { //~ depends: Container@6, Numbers@12
    type Item = i32;

    fn first(&self) -> Self::Item { //~ depends: first@9, Numbers@12, Item@15
        1
    }
}

fn main() {
    let n = Numbers; //~ depends: Numbers@12
    let _ = n.first(); //~ depends: first@17, n@23
}
