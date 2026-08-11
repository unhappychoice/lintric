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

// A binding is not among the candidates for its own initializer, so `let w = w + 1` reads the
// previous `w` — including when that one is in an enclosing scope, and when the initializer is on a
// line of its own.
fn shadowing() -> i32 {
    let w = 1;
    {
        let w = w + 1; //~ depends: w@27
        let w =
            w + 1; //~ depends: w@29
        w //~ depends: w@30
    }
}

// A closure parameter and a generic are declared on the same line as the body reading them, and both
// are visible there — which is why the rule above is about the initializer rather than the line.
fn same_line() -> i32 {
    let x = 100;
    let closure = |x: i32| x + 1;
    closure(x) //~ depends: closure@40, x@39
}
