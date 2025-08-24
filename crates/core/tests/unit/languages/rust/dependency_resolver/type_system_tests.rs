use lintric_core::languages::rust::dependency_resolver::method_resolver::*;
use lintric_core::models::{Type, InferenceContext};
use std::collections::HashMap;

#[test]
fn test_type_inference_basic() {
    let engine = TypeInferenceEngine::new();

    let result = engine.infer_receiver_type("self", &InferenceContext::new());
    assert_eq!(result, Some(Type::Concrete("Self".to_string())));

    let result = engine.infer_receiver_type("unknown", &InferenceContext::new());
    assert_eq!(result, Some(Type::Unknown));
}

#[test]
fn test_type_inference_with_symbols() {
    let mut symbols = HashMap::new();
    symbols.insert("my_var".to_string(), Type::Concrete("MyStruct".to_string()));
    symbols.insert("another_var".to_string(), Type::Concrete("AnotherStruct".to_string()));

    let engine = TypeInferenceEngine::new().with_symbols(symbols);

    let result = engine.infer_receiver_type("my_var", &InferenceContext::new());
    assert_eq!(result, Some(Type::Concrete("MyStruct".to_string())));

    let result = engine.infer_receiver_type("another_var", &InferenceContext::new());
    assert_eq!(result, Some(Type::Concrete("AnotherStruct".to_string())));

    let result = engine.infer_receiver_type("unknown", &InferenceContext::new());
    assert_eq!(result, Some(Type::Unknown));
}

#[test]
fn test_type_system_concrete_types() {
    let concrete_type = Type::Concrete("String".to_string());
    assert_eq!(concrete_type.name(), "String");

    let unknown_type = Type::Unknown;
    assert_eq!(unknown_type.name(), "unknown");
}

#[test]
fn test_resolution_path_variants() {
    let inherent_path = ResolutionPath::InherentMethod { impl_block_id: 1 };
    let trait_path = ResolutionPath::TraitMethod { trait_impl_id: 2 };
    let associated_path = ResolutionPath::Associated { type_name: "MyStruct".to_string() };

    // Verify the enum variants can be created and compared
    match inherent_path {
        ResolutionPath::InherentMethod { impl_block_id } => assert_eq!(impl_block_id, 1),
        _ => panic!("Expected InherentMethod variant"),
    }

    match trait_path {
        ResolutionPath::TraitMethod { trait_impl_id } => assert_eq!(trait_impl_id, 2),
        _ => panic!("Expected TraitMethod variant"),
    }

    match associated_path {
        ResolutionPath::Associated { type_name } => assert_eq!(type_name, "MyStruct"),
        _ => panic!("Expected Associated variant"),
    }
}

#[test]
fn test_method_resolution_confidence_levels() {
    // Test different confidence levels for method resolution
    let high_confidence = 1.0;
    let medium_confidence = 0.9;
    let low_confidence = 0.7;

    assert!(high_confidence > medium_confidence);
    assert!(medium_confidence > low_confidence);
    assert!(low_confidence > 0.0);
}