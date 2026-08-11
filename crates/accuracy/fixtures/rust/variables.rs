// Let bindings, arithmetic and shadowing.

fn main() {
    let a = 1;
    let b = a + 2; //~ depends: a@4
    let c = a + b; //~ depends: a@4, b@5
    let d = c; //~ depends: c@6
    let a = d + 1; //~ depends: d@7
    let e = a; //~ depends: a@8
    let _ = e; //~ depends: e@9
}
