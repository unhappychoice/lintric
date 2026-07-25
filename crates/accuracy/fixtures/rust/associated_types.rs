// Associated types in traits and their implementations.
//
// Whether `Self::Item` inside the impl should name the impl's own alias rather than the trait's
// declaration is unsettled; these annotations follow the trait declaration.

trait Container {
    type Item;

    fn first(&self) -> Self::Item; //~ depends: Container@6, Item@7
}

struct Numbers;

impl Container for Numbers { //~ depends: Container@6, Numbers@12
    type Item = i32;

    fn first(&self) -> Self::Item { //~ depends: first@9, Container@6, Item@7
        1
    }
}

fn main() {
    let n = Numbers; //~ depends: Numbers@12
    let _ = n.first(); //~ depends: first@17, n@23
}
