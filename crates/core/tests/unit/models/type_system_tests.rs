use lintric_core::models::{InferenceContext, Type};

#[test]
fn test_type_creation() {
    let concrete_type = Type::Concrete("String".to_string());
    let generic_type = Type::Generic("T".to_string(), vec![]);
    let unknown_type = Type::Unknown;

    assert!(matches!(concrete_type, Type::Concrete(_)));
    assert!(matches!(generic_type, Type::Generic(_, _)));
    assert!(matches!(unknown_type, Type::Unknown));
}

#[test]
fn test_type_name() {
    let string_type = Type::Concrete("String".to_string());
    let generic_type = Type::Generic("T".to_string(), vec![]);
    let unknown_type = Type::Unknown;

    assert_eq!(string_type.name(), "String");
    assert_eq!(generic_type.name(), "T");
    assert_eq!(unknown_type.name(), "unknown");
}

#[test]
fn test_type_is_generic() {
    let concrete_type = Type::Concrete("i32".to_string());
    let generic_type = Type::Generic("T".to_string(), vec![]);
    let type_param = Type::TypeParameter("U".to_string());
    let unknown_type = Type::Unknown;

    assert!(!concrete_type.is_generic());
    assert!(generic_type.is_generic());
    assert!(!type_param.is_generic());
    assert!(!unknown_type.is_generic());
}

#[test]
fn test_type_is_type_parameter() {
    let concrete_type = Type::Concrete("i32".to_string());
    let generic_type = Type::Generic("T".to_string(), vec![]);
    let type_param = Type::TypeParameter("U".to_string());
    let unknown_type = Type::Unknown;

    assert!(!concrete_type.is_type_parameter());
    assert!(!generic_type.is_type_parameter());
    assert!(type_param.is_type_parameter());
    assert!(!unknown_type.is_type_parameter());
}

#[test]
fn test_type_is_reference() {
    let concrete_type = Type::Concrete("i32".to_string());
    let reference_type = Type::Reference(Box::new(Type::Concrete("i32".to_string())));

    assert!(!concrete_type.is_reference());
    assert!(reference_type.is_reference());
}

#[test]
fn test_type_deref() {
    let inner_type = Type::Concrete("i32".to_string());
    let reference_type = Type::Reference(Box::new(inner_type.clone()));

    assert_eq!(reference_type.deref(), inner_type);
    assert_eq!(inner_type.deref(), inner_type);
}

#[test]
fn test_type_equality() {
    let string1 = Type::Concrete("String".to_string());
    let string2 = Type::Concrete("String".to_string());
    let int_type = Type::Concrete("i32".to_string());
    let generic1 = Type::Generic("T".to_string(), vec![]);
    let generic2 = Type::Generic("T".to_string(), vec![]);
    let unknown1 = Type::Unknown;
    let unknown2 = Type::Unknown;

    assert_eq!(string1, string2);
    assert_ne!(string1, int_type);
    assert_eq!(generic1, generic2);
    assert_eq!(unknown1, unknown2);
    assert_ne!(string1, generic1);
    assert_ne!(generic1, unknown1);
}

#[test]
fn test_type_substitute_type_parameter() {
    let type_param = Type::TypeParameter("T".to_string());
    let concrete_type = Type::Concrete("i32".to_string());

    let substituted = type_param.substitute_type_parameter("T", &concrete_type);
    assert_eq!(substituted, concrete_type);

    let non_matching = type_param.substitute_type_parameter("U", &concrete_type);
    assert_eq!(non_matching, type_param);
}

#[test]
fn test_inference_context_creation() {
    let context = InferenceContext::new();

    // Test that context can be created
    assert!(context.symbol_table.is_empty());
    assert!(context.type_cache.is_empty());
}

#[test]
fn test_inference_context_symbol_operations() {
    let mut context = InferenceContext::new();

    context.add_symbol("x".to_string(), Type::Concrete("i32".to_string()));
    context.add_symbol("y".to_string(), Type::Generic("T".to_string(), vec![]));

    assert_eq!(
        context.get_symbol_type("x"),
        Some(&Type::Concrete("i32".to_string()))
    );
    assert_eq!(
        context.get_symbol_type("y"),
        Some(&Type::Generic("T".to_string(), vec![]))
    );
    assert_eq!(context.get_symbol_type("z"), None);
}

#[test]
fn test_inference_context_type_cache() {
    let mut context = InferenceContext::new();

    context.cache_type(1, Type::Concrete("String".to_string()));
    context.cache_type(2, Type::Unknown);

    assert_eq!(
        context.get_cached_type(1),
        Some(&Type::Concrete("String".to_string()))
    );
    assert_eq!(context.get_cached_type(2), Some(&Type::Unknown));
    assert_eq!(context.get_cached_type(3), None);
}

#[test]
fn test_inference_context_clone() {
    let mut original = InferenceContext::new();
    original.add_symbol("test".to_string(), Type::Concrete("i32".to_string()));
    original.cache_type(1, Type::Unknown);

    let cloned = original.clone();

    assert_eq!(original.symbol_table, cloned.symbol_table);
    assert_eq!(original.type_cache, cloned.type_cache);
}
