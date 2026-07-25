// Closures capturing bindings from the enclosing scope.

fn main() {
    let base = 10;
    let add = |n: i32| {
        n + base //~ depends: n@5, base@4
    };
    let result = add(5); //~ depends: add@5
    let nested = |n: i32| {
        add(n) + base //~ depends: add@5, n@9, base@4
    };
    let total = result + nested(1); //~ depends: result@8, nested@9
    let _ = total; //~ depends: total@12
}
