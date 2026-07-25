// Tuple structs, unit structs and their construction.

struct Meters(i32);

struct Marker;

struct Wrapper(Meters, Marker); //~ depends: Meters@3, Marker@5

fn unwrap(w: Wrapper) -> i32 { //~ depends: Wrapper@7
    let Wrapper(meters, _) = w; //~ depends: Wrapper@7, w@9
    let Meters(value) = meters; //~ depends: Meters@3, meters@10
    value //~ depends: value@11
}

fn main() {
    let w = Wrapper(Meters(1), Marker); //~ depends: Wrapper@7, Meters@3, Marker@5
    let _ = unwrap(w); //~ depends: unwrap@9, w@16
}
