use lintric_core::{analyze_content, Language};

#[test]
fn each_generic_item_resolves_its_own_type_parameter() {
    let source =
        "trait A<T> {\n    fn a(&self, v: T);\n}\n\ntrait B<T> {\n    fn b(&self, v: T);\n}\n";
    let dependencies = type_dependencies(source);

    assert!(
        dependencies.contains(&(2, 1)),
        "A's own T: {dependencies:?}"
    );
    assert!(
        dependencies.contains(&(6, 5)),
        "B's own T: {dependencies:?}"
    );
    assert!(
        !dependencies.contains(&(6, 1)),
        "B must not reach A's T: {dependencies:?}"
    );
}

#[test]
fn a_type_parameter_used_on_its_own_declaration_line_produces_no_edge() {
    let source = "trait A<T> {\n    fn a(&self, v: T);\n}\n\nfn f<T>(v: T) -> T {\n    v\n}\n";
    let dependencies = type_dependencies(source);

    assert!(
        !dependencies.iter().any(|(from, _)| *from == 5),
        "line 5 declares and uses T, so no edge: {dependencies:?}"
    );
}

#[test]
fn a_supertrait_bound_names_the_local_parameter() {
    let source = "trait Base<T> {\n    fn run(&self, v: T);\n}\n\ntrait Extended<T>: Base<T> {}\n";
    let dependencies = type_dependencies(source);

    assert!(
        !dependencies.iter().any(|(from, _)| *from == 5),
        "T on line 5 is Extended's own: {dependencies:?}"
    );
}

#[test]
fn a_module_local_type_wins_over_a_top_level_import_of_the_same_name() {
    let source = "mod geometry {\n    pub struct Rect;\n\n    pub fn unit() -> Rect {\n        Rect\n    }\n}\n\nuse geometry::Rect;\n";
    let (ir, _) = analyze_content(source.to_string(), Language::Rust).unwrap();

    let inside: Vec<usize> = ir
        .dependencies
        .iter()
        .filter(|d| d.source_line == 4 && d.symbol == "Rect")
        .map(|d| d.target_line)
        .collect();

    assert_eq!(inside, vec![2], "the module's own struct, not the import");
}

#[test]
fn an_impl_reaches_its_own_associated_type_before_the_traits() {
    let source = "trait Container {\n    type Item;\n\n    fn first(&self) -> Self::Item;\n}\n\nstruct N;\n\nimpl Container for N {\n    type Item = i32;\n\n    fn first(&self) -> Self::Item {\n        1\n    }\n}\n";
    let dependencies = type_dependencies(source);

    assert!(
        dependencies.contains(&(12, 10)),
        "the impl's own alias: {dependencies:?}"
    );
}

fn type_dependencies(source: &str) -> Vec<(usize, usize)> {
    let (ir, _) = analyze_content(source.to_string(), Language::Rust).unwrap();

    ir.dependencies
        .iter()
        .filter(|dependency| dependency.symbol == "T" || dependency.symbol == "Item")
        .map(|dependency| (dependency.source_line, dependency.target_line))
        .collect()
}
