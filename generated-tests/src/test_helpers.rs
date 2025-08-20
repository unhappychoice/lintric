pub fn generate_helper_functions() -> String {
    r#"fn assert_code_analysis_and_snapshot(
    source_code: &str,
    language: lintric_core::Language,
    snapshot_folder: &str,
    test_name: &str,
    error_msg: &str,
    node_validation_fn: impl FnOnce(&str),
) {
    let mut settings = insta::Settings::clone_current();
    settings.set_snapshot_path(format!("snapshots/{}", snapshot_folder));
    let _guard = settings.bind_to_scope();

    let result = std::panic::catch_unwind(|| {
        lintric_core::analyze_content(source_code.to_string(), language.clone())
    });

    assert!(result.is_ok(), "{}", error_msg);

    if let Ok(Ok((ir, _metrics))) = result {
        // Get AST for comprehensive snapshot
        let ast_result =
            lintric_core::get_s_expression_from_content(source_code.to_string(), language.clone());
        let ast_string = ast_result.unwrap_or_else(|e| format!("AST Error: {}", e));

        // Create comprehensive snapshot content
        let snapshot_content = format!(
            "Source Code:\n{}\n\nAST:\n{}\n\nIR:\n{:#?}",
            source_code, ast_string, ir
        );

        insta::assert_snapshot!(test_name, snapshot_content);

        // Verify that the generated code contains the expected node type
        let s_expr_result = std::panic::catch_unwind(|| {
            lintric_core::get_s_expression_from_content(source_code.to_string(), language)
        });

        if let Ok(Ok(s_expr)) = s_expr_result {
            node_validation_fn(&s_expr);
        }
    }
}"#
    .to_string()
}
