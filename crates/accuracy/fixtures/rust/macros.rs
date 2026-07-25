// Macro invocations, including inline format string captures.

fn main() {
    let name = "lintric";
    let count = 3;
    let msg = format!("{name} x{count}"); //~ depends: name@4, count@5
    println!("{msg}"); //~ depends: msg@6
    println!("{} {}", name, count); //~ depends: name@4, count@5
    let items = vec![name, msg.as_str()]; //~ depends: name@4, msg@6
    assert_eq!(items.len(), 2); //~ depends: items@9
}
