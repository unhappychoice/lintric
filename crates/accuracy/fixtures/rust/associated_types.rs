// Associated types in traits and their implementations.
//
// `Self::Item` inside the impl names the impl's own alias, which is nearer than the trait's
// declaration. `Self` itself is the type inside an impl and the trait inside a trait.
//
// The impl's alias also references the trait's declaration, the same way an implemented method
// references the signature it satisfies.

trait Container {
    type Item;

    fn first(&self) -> Self::Item; //~ depends: Container@9, Item@10
}

struct Numbers;

impl Container for Numbers { //~ depends: Container@9, Numbers@15
    type Item = i32; //~ depends: Item@10

    fn first(&self) -> Self::Item { //~ depends: first@12, Numbers@15, Item@18
        1
    }
}

fn main() {
    let n = Numbers; //~ depends: Numbers@15
    let _ = n.first(); //~ depends: first@20, n@26
}
