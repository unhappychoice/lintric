// Function definitions, parameters and nested calls.

fn double(n: i32) -> i32 {
    n * 2 //~ depends: n@3
}

fn quadruple(n: i32) -> i32 {
    double(double(n)) //~ depends: double@3, n@7
}

fn main() {
    let x = 3;
    let y = quadruple(x); //~ depends: quadruple@7, x@12
    let z = double(y); //~ depends: double@3, y@13
    let _ = z; //~ depends: z@14
}
