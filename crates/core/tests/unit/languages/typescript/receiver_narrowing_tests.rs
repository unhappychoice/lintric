use lintric_core::{analyze_content, Language};

/// Two types declaring `label`, on lines 2 and 5, so a member edge names which one it reached.
const SHARED_MEMBER: &str =
    "interface Reader {\n    label: string;\n}\nclass Writer {\n    label: number;\n}\n";

const READER_LABEL: usize = 2;
const WRITER_LABEL: usize = 5;

#[test]
fn an_annotated_parameter_reaches_only_its_own_types_member() {
    let source = format!(
        "{SHARED_MEMBER}\nfunction f(reader: Reader): string {{\n    return reader.label;\n}}\n"
    );
    let reached = members_reached(&source);

    assert_eq!(reached, vec![READER_LABEL], "{:?}", dependencies(&source));
}

#[test]
fn an_annotated_variable_reaches_only_its_own_types_member() {
    let source =
        format!("{SHARED_MEMBER}\nfunction f(): number {{\n    const w: Writer = new Writer();\n    return w.label;\n}}\n");
    let reached = members_reached(&source);

    assert_eq!(reached, vec![WRITER_LABEL], "{:?}", dependencies(&source));
}

#[test]
fn this_reaches_the_member_of_the_class_it_sits_in() {
    let source = "interface Reader {\n    label: string;\n}\nclass Writer {\n    label: number;\n\n    read(): number {\n        return this.label;\n    }\n}\n";
    let reached = members_reached(source);

    assert_eq!(reached, vec![WRITER_LABEL], "{:?}", dependencies(source));
}

#[test]
fn a_union_annotation_reaches_the_member_of_every_type_it_names() {
    let source = format!(
        "{SHARED_MEMBER}\nfunction f(either: Reader | Writer): void {{\n    void either.label;\n}}\n"
    );
    let reached = members_reached(&source);

    assert_eq!(
        reached,
        vec![READER_LABEL, WRITER_LABEL],
        "{:?}",
        dependencies(&source)
    );
}

#[test]
fn an_unannotated_receiver_reaches_neither_rather_than_one_of_them() {
    // Picking either would claim a relationship with a type this file never states.
    let source =
        format!("{SHARED_MEMBER}\nfunction f(untyped): void {{\n    void untyped.label;\n}}\n");

    assert!(
        members_reached(&source).is_empty(),
        "{:?}",
        dependencies(&source)
    );
}

#[test]
fn a_member_declared_once_still_resolves_without_an_annotation() {
    // Nothing to tell apart, so the receiver's type need not be known.
    let source = "interface Reader {\n    label: string;\n}\nfunction f(untyped): void {\n    void untyped.label;\n}\n";

    assert_eq!(members_reached(source), vec![READER_LABEL]);
}

/// The declaration lines that member edges named `label` point at.
fn members_reached(source: &str) -> Vec<usize> {
    let mut reached: Vec<usize> = dependencies(source)
        .iter()
        .filter(|(_, target, symbol)| {
            symbol == "label" && [READER_LABEL, WRITER_LABEL].contains(target)
        })
        .map(|(_, target, _)| *target)
        .collect();

    reached.sort_unstable();
    reached.dedup();
    reached
}

fn dependencies(source: &str) -> Vec<(usize, usize, String)> {
    let (ir, _) = analyze_content(source.to_string(), Language::TypeScript).unwrap();

    ir.dependencies
        .iter()
        .map(|dependency| {
            (
                dependency.source_line,
                dependency.target_line,
                dependency.symbol.clone(),
            )
        })
        .collect()
}
