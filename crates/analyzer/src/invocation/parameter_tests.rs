use mago_codex::metadata::flags::MetadataFlags;
use mago_codex::metadata::function_like::FunctionLikeKind;
use mago_codex::metadata::ttype::TypeMetadata;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::get_int;
use mago_codex::ttype::get_null;
use mago_span::Span;
use mago_word::word;

use super::*;

#[test]
fn bound_closure_parameters_preserve_declaration_metadata() {
    let span = Span::dummy(0, 1);
    let named = |name: &str| TUnion::from_atomic(TAtomic::Object(TObject::new_named(word(name))));
    let mut parameter = FunctionLikeParameterMetadata::new(
        VariableIdentifier(word("$item")),
        span,
        span,
        MetadataFlags::BY_REFERENCE | MetadataFlags::HAS_DEFAULT,
    );
    parameter.type_metadata = Some(TypeMetadata::new(named("parent"), span));
    parameter.out_type = Some(TypeMetadata::new(get_int(), span));
    parameter.default_type = Some(TypeMetadata::new(get_null(), span));
    parameter.closure_this_type = Some(TypeMetadata::new(named("Receiver"), span));
    let metadata = FunctionLikeMetadata::new(
        FunctionLikeKind::Closure,
        word("callback"),
        word("callback"),
        span,
        MetadataFlags::empty(),
    )
    .with_parameters([parameter]);
    let signature = TCallableSignature::new(false, true).with_parameters(vec![
        TCallableParameter::new(Some(Arc::new(named("Base"))), true, false, true)
            .with_name(Some(VariableIdentifier(word("$item")))),
    ]);
    let mut target = InvocationTarget::FunctionLike {
        identifier: FunctionLikeIdentifier::Closure(metadata.name),
        metadata: &metadata,
        inferred_return_type: None,
        effective_signature: None,
        method_context: None,
        span,
    };
    target.bind_closure_parameters(&signature);
    let Some(parameter) = target.get_parameter(0) else { panic!("missing bound parameter") };
    assert_eq!(parameter.get_type(), Some(&named("Base")));
    assert_eq!(parameter.get_out_type(), Some(&get_int()));
    assert_eq!(parameter.get_default_type(), Some(&get_null()));
    assert_eq!(parameter.get_closure_this_type(), Some(&named("Receiver")));
    assert_eq!(parameter.get_name(), Some(&VariableIdentifier(word("$item"))));
    assert!(parameter.is_by_reference());
    assert!(parameter.has_default());
    assert!(!parameter.is_variadic());
    assert!(target.allows_named_arguments());

    let signature = TCallableSignature::new(false, true).with_parameters(vec![
        TCallableParameter::new(Some(Arc::new(named("parent"))), true, false, true)
            .with_out_type(Some(Arc::new(named("Base"))))
            .with_closure_this_type(Some(Arc::new(named("BoundReceiver")))),
    ]);
    target.bind_closure_parameters(&signature);
    let Some(parameter) = target.get_parameter(0) else { panic!("missing parameter with bound effects") };
    assert_eq!(parameter.get_out_type(), Some(&named("Base")));
    assert_eq!(parameter.get_closure_this_type(), Some(&named("BoundReceiver")));
    assert_eq!(parameter.get_default_type(), Some(&get_null()));
    assert!(parameter.is_by_reference());
}
