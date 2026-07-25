// Nested blocks, shadowing and nested function items.

fn main() {
    let outer = 1;
    let shadowed = 2;
    {
        let inner = outer + 1; //~ depends: outer@4
        let shadowed = inner; //~ depends: inner@7
        let _ = shadowed; //~ depends: shadowed@8
    }
    // The block-scoped binding is out of scope again, so this reads the outer one.
    let _ = shadowed; //~ depends: shadowed@5
}

fn nesting() -> i32 {
    fn inner() -> i32 {
        1
    }

    inner() //~ depends: inner@16
}
