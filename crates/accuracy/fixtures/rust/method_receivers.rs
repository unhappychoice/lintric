// Method calls told apart by the receiver's type.
//
// Two types declaring a method of the same name are distinguished by what the receiver is: a
// parameter or a `let` whose type the file states, or `self` inside an impl. Where the file does not
// state it, the call is left unresolved rather than pointed at both declarations. See #254.

struct First;

struct Second;

impl First { //~ depends: First@7
    fn value(&self) -> i32 {
        1
    }

    fn doubled(&self) -> i32 {
        self.value() * 2 //~ depends: value@12
    }
}

impl Second { //~ depends: Second@9
    fn value(&self) -> i32 {
        2
    }
}

fn from_parameter(f: First) -> i32 { //~ depends: First@7
    f.value() //~ depends: f@27, value@12
}

fn from_annotated_binding(s: Second) -> i32 { //~ depends: Second@9
    let bound: Second = s; //~ depends: Second@9, s@31
    bound.value() //~ depends: bound@32, value@22
}

fn from_initializer() -> i32 {
    let made = First; //~ depends: First@7
    made.value() //~ depends: made@37, value@12
}

fn through_a_borrow(s: &Second) -> i32 { //~ depends: Second@9
    (*s).value() //~ depends: s@41, value@22
}
