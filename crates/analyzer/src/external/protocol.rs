//! Stable binary messages for worker-backed analyzer providers.

#![allow(clippy::big_endian_bytes, reason = "network byte order is part of the stable analyzer wire format")]

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use foldhash::HashSet;
use foldhash::fast::RandomState;
use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::misc::GenericParent;
use mago_codex::misc::VariableIdentifier;
use mago_codex::ttype::TType;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::alias::TAlias;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::key::ArrayKey;
use mago_codex::ttype::atomic::array::keyed::TKeyedArray;
use mago_codex::ttype::atomic::array::list::TList;
use mago_codex::ttype::atomic::callable::TCallable;
use mago_codex::ttype::atomic::callable::TCallableConstraint;
use mago_codex::ttype::atomic::callable::TCallableSignature;
use mago_codex::ttype::atomic::callable::parameter::TCallableParameter;
use mago_codex::ttype::atomic::conditional::TConditional;
use mago_codex::ttype::atomic::derived::TDerived;
use mago_codex::ttype::atomic::derived::index_access::TIndexAccess;
use mago_codex::ttype::atomic::derived::int_mask::TIntMask;
use mago_codex::ttype::atomic::derived::int_mask_of::TIntMaskOf;
use mago_codex::ttype::atomic::derived::intersection::TDerivedIntersection;
use mago_codex::ttype::atomic::derived::key_of::TKeyOf;
use mago_codex::ttype::atomic::derived::new::TNew;
use mago_codex::ttype::atomic::derived::properties_of::TPropertiesOf;
use mago_codex::ttype::atomic::derived::template_type::TTemplateType;
use mago_codex::ttype::atomic::derived::value_of::TValueOf;
use mago_codex::ttype::atomic::generic::TGenericParameter;
use mago_codex::ttype::atomic::iterable::TIterable;
use mago_codex::ttype::atomic::mixed::TMixed;
use mago_codex::ttype::atomic::mixed::truthiness::TMixedTruthiness;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::has_method::TObjectHasMethod;
use mago_codex::ttype::atomic::object::has_property::TObjectHasProperty;
use mago_codex::ttype::atomic::object::named::TNamedObject;
use mago_codex::ttype::atomic::object::with_properties::TObjectWithProperties;
use mago_codex::ttype::atomic::reference::TGlobalReferenceSelector;
use mago_codex::ttype::atomic::reference::TReference;
use mago_codex::ttype::atomic::reference::TReferenceMemberSelector;
use mago_codex::ttype::atomic::resource::TResource;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::bool::TBool;
use mago_codex::ttype::atomic::scalar::class_like_string::TClassLikeString;
use mago_codex::ttype::atomic::scalar::class_like_string::TClassLikeStringKind;
use mago_codex::ttype::atomic::scalar::float::TFloat;
use mago_codex::ttype::atomic::scalar::int::TInteger;
use mago_codex::ttype::atomic::scalar::string::TString;
use mago_codex::ttype::atomic::scalar::string::TStringCasing;
use mago_codex::ttype::atomic::scalar::string::TStringLiteral;
use mago_codex::ttype::comparator::ComparisonResult;
use mago_codex::ttype::comparator::union_comparator;
use mago_codex::ttype::get_bool;
use mago_codex::ttype::get_false;
use mago_codex::ttype::get_float;
use mago_codex::ttype::get_int;
use mago_codex::ttype::get_keyed_array;
use mago_codex::ttype::get_list;
use mago_codex::ttype::get_literal_string;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::get_never;
use mago_codex::ttype::get_non_empty_string;
use mago_codex::ttype::get_non_negative_int;
use mago_codex::ttype::get_object;
use mago_codex::ttype::get_string;
use mago_codex::ttype::get_true;
use mago_codex::ttype::template::variance::Variance;
use mago_codex::ttype::union::TUnion;
use mago_codex::visibility::Visibility;
use mago_database::file::File;
use mago_extension::PayloadReader;
use mago_extension::PayloadWriter;
use mago_php_version::PHPVersion;
use mago_span::HasSpan;
use mago_word::word;

use crate::artifacts::AnalysisArtifacts;
use crate::external::ExternalAnalysisSession;
use crate::external::ExternalExtension;
use crate::external::ExternalPlugin;
use crate::external::FunctionProvider;
use crate::external::FunctionTarget;
use crate::external::MethodProvider;
use crate::external::MethodTarget;
use crate::external::error::ExternalAnalyzerError;
use crate::external::error::protocol;
use crate::external::metadata;
use crate::invocation::Invocation;

pub const ANALYZER_PROTOCOL_MAGIC: [u8; 4] = *b"MANA";
pub const ANALYZER_PROTOCOL_MAJOR: u16 = 1;
pub const ANALYZER_PROTOCOL_MINOR: u16 = 0;

const HEADER_LENGTH: usize = 12;
const INITIAL_MESSAGE_CAPACITY: usize = 256;
const DESCRIBE_REQUEST: u16 = 1;
const RETURN_TYPE_REQUEST: u16 = 2;
const TYPE_COMPARISON_REQUEST: u16 = 3;
const INITIALIZE_REQUEST: u16 = 11;
const DESCRIBE_RESPONSE: u16 = 0x8001;
const RETURN_TYPE_RESPONSE: u16 = 0x8002;
const TYPE_COMPARISON_RESPONSE: u16 = 0x8003;
const INITIALIZE_RESPONSE: u16 = 0x800B;
const MAXIMUM_EXTENSIONS: usize = 0x4000;
const MAXIMUM_PLUGINS: usize = 0x4000;
const MAXIMUM_PROVIDERS: usize = 0x0001_0000;
const MAXIMUM_TARGETS: usize = 0x0001_0000;
const MAXIMUM_ALIASES: usize = 256;
const MAXIMUM_STUBS: usize = 0x0001_0000;
// Flow-sensitive inference can legitimately build deeply nested shapes after
// repeated writes. Keep a finite protocol guard, but leave enough room for
// complete snapshots from real framework codebases such as Symfony.
const MAXIMUM_TYPE_DEPTH: usize = 256;
const MAXIMUM_TYPE_MEMBERS: usize = 0x0001_0000;

const TARGET_EXACT: u8 = 1;
const TARGET_PREFIX: u8 = 2;
const TARGET_NAMESPACE: u8 = 3;
const INVOCATION_FUNCTION: u8 = 1;
const INVOCATION_METHOD: u8 = 2;

const TYPE_REFERENCE: u8 = 0;
const TYPE_MIXED: u8 = 1;
const TYPE_NEVER: u8 = 2;
const TYPE_NULL: u8 = 3;
const TYPE_VOID: u8 = 4;
const TYPE_BOOL: u8 = 5;
const TYPE_TRUE: u8 = 6;
const TYPE_FALSE: u8 = 7;
const TYPE_INT: u8 = 8;
const TYPE_FLOAT: u8 = 9;
const TYPE_STRING: u8 = 10;
const TYPE_LITERAL_STRING: u8 = 11;
const TYPE_OBJECT: u8 = 12;
const TYPE_NAMED_OBJECT: u8 = 13;
const TYPE_ARRAY: u8 = 14;
const TYPE_LIST: u8 = 15;
const TYPE_UNION: u8 = 16;
const TYPE_NON_NEGATIVE_INT: u8 = 17;
const TYPE_NON_EMPTY_STRING: u8 = 18;
const TYPE_LITERAL_INT: u8 = 19;
const TYPE_COMPLETE: u8 = 20;

const TYPE_COMPARISON_EQUAL: u8 = 1;
const TYPE_COMPARISON_CONTAINED_BY: u8 = 2;
const TYPE_COMPARISON_CAN_BE_IDENTICAL: u8 = 3;

const SNAPSHOT_SCALAR: u8 = 1;
const SNAPSHOT_CALLABLE: u8 = 2;
const SNAPSHOT_MIXED: u8 = 3;
const SNAPSHOT_OBJECT: u8 = 4;
const SNAPSHOT_ARRAY: u8 = 5;
const SNAPSHOT_ITERABLE: u8 = 6;
const SNAPSHOT_RESOURCE: u8 = 7;
const SNAPSHOT_REFERENCE: u8 = 8;
const SNAPSHOT_GENERIC_PARAMETER: u8 = 9;
const SNAPSHOT_VARIABLE: u8 = 10;
const SNAPSHOT_CONDITIONAL: u8 = 11;
const SNAPSHOT_DERIVED: u8 = 12;
const SNAPSHOT_ALIAS: u8 = 13;
const SNAPSHOT_NEVER: u8 = 14;
const SNAPSHOT_NULL: u8 = 15;
const SNAPSHOT_VOID: u8 = 16;
const SNAPSHOT_PLACEHOLDER: u8 = 17;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Registration {
    pub extensions: Vec<ExternalExtension>,
    pub plugins: Vec<ExternalPlugin>,
    pub function_providers: Vec<FunctionProvider>,
    pub method_providers: Vec<MethodProvider>,
    pub initialization_plugins: Vec<u16>,
    pub before_analysis_plugins: Vec<u16>,
    pub after_file_analysis_plugins: Vec<u16>,
    pub after_analysis_plugins: Vec<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct InitializationStub {
    pub plugin: u16,
    pub filename: Vec<u8>,
    pub contents: Vec<u8>,
}

pub(super) struct ReturnTypeRequest<'type_info> {
    pub payload: Vec<u8>,
    pub types: Vec<&'type_info TUnion>,
    pub arguments: usize,
    pub typed_arguments: usize,
    pub type_snapshot_duration: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum NestedRequestKind {
    TypeComparison,
    CodebaseQuery,
    AnalysisQuery,
}

pub(super) fn encode_describe_request(php_version: PHPVersion) -> Vec<u8> {
    let mut writer = message_writer(DESCRIBE_REQUEST);
    writer.write_u32(php_version.to_version_id());
    writer.finish()
}

pub(super) fn encode_initialization_request(plugins: &[u16]) -> Result<Vec<u8>, ExternalAnalyzerError> {
    let mut writer = message_writer(INITIALIZE_REQUEST);
    writer.write_length(plugins.len())?;
    for plugin in plugins {
        writer.write_u16(*plugin);
    }

    Ok(writer.finish())
}

pub(super) fn decode_initialization_response(
    payload: &[u8],
    expected_plugins: &[u16],
) -> Result<Vec<InitializationStub>, ExternalAnalyzerError> {
    let mut reader = message_reader(payload, INITIALIZE_RESPONSE)?;
    let plugin_count = reader.read_count("initialized plugins", MAXIMUM_PLUGINS)?;
    if plugin_count != expected_plugins.len() {
        return Err(protocol(format!(
            "worker initialized {plugin_count} plugins, but Mago requested {}",
            expected_plugins.len()
        )));
    }

    let mut stubs = Vec::new();
    for expected_plugin in expected_plugins {
        let plugin = reader.read_u16("initialized plugin index")?;
        if plugin != *expected_plugin {
            return Err(protocol(format!(
                "worker initialized plugin index {plugin}, but Mago expected {expected_plugin}"
            )));
        }

        let stub_count = reader.read_count("external stubs", MAXIMUM_STUBS)?;
        let mut filenames = HashSet::with_capacity_and_hasher(stub_count, RandomState::default());
        for _ in 0..stub_count {
            let filename = non_empty(reader.read_bytes("external stub filename")?.to_vec(), "external stub filename")?;
            if filename.contains(&0) {
                return Err(protocol("external stub filename contains NUL"));
            }

            if !filenames.insert(filename.clone()) {
                return Err(protocol(format!(
                    "plugin {plugin} returned external stub `{}` more than once",
                    String::from_utf8_lossy(&filename)
                )));
            }

            let contents = reader.read_bytes("external stub contents")?.to_vec();
            stubs.push(InitializationStub { plugin, filename, contents });
        }
    }

    reader.finish()?;
    Ok(stubs)
}

pub(super) fn decode_registration(payload: &[u8]) -> Result<Registration, ExternalAnalyzerError> {
    let mut reader = message_reader(payload, DESCRIBE_RESPONSE)?;
    let extension_count = reader.read_count("extensions", MAXIMUM_EXTENSIONS)?;
    if extension_count == 0 {
        return Err(protocol("worker registration contains no extensions"));
    }

    let mut extensions = Vec::with_capacity(extension_count);
    let mut plugins = Vec::new();
    let mut function_providers = Vec::new();
    let mut method_providers = Vec::new();
    let mut initialization_plugins = Vec::new();
    let mut before_analysis_plugins = Vec::new();
    let mut after_file_analysis_plugins = Vec::new();
    let mut after_analysis_plugins = Vec::new();
    for _ in 0..extension_count {
        let extension_identifier = non_empty(reader.read_string("extension identifier")?, "extension identifier")?;
        let extension_name = non_empty(reader.read_string("extension name")?, "extension name")?;
        let extension_version = non_empty(reader.read_string("extension version")?, "extension version")?;
        let plugin_count = reader.read_count("plugins", MAXIMUM_PLUGINS)?;
        let mut extension_plugins = Vec::with_capacity(plugin_count);
        for _ in 0..plugin_count {
            let identifier = non_empty(reader.read_string("plugin identifier")?, "plugin identifier")?;
            let name = non_empty(reader.read_string("plugin name")?, "plugin name")?;
            let description = non_empty(reader.read_string("plugin description")?, "plugin description")?;
            let default_enabled = reader.read_bool("plugin default-enabled flag")?;
            let lifecycle = reader.read_u8("plugin lifecycle flags")?;
            if lifecycle & !0b1111 != 0 {
                return Err(protocol(format!("plugin `{identifier}` has unknown lifecycle flags {lifecycle:#04x}")));
            }
            let index =
                u16::try_from(plugins.len()).map_err(|_| protocol("worker registered more than 65,536 plugins"))?;
            if lifecycle & 1 != 0 {
                before_analysis_plugins.push(index);
            }
            if lifecycle & 2 != 0 {
                after_file_analysis_plugins.push(index);
            }
            if lifecycle & 4 != 0 {
                after_analysis_plugins.push(index);
            }

            if lifecycle & 8 != 0 {
                initialization_plugins.push(index);
            }

            let alias_count = reader.read_count("plugin aliases", MAXIMUM_ALIASES)?;
            let mut aliases = Vec::with_capacity(alias_count);
            for _ in 0..alias_count {
                aliases.push(non_empty(reader.read_string("plugin alias")?, "plugin alias")?);
            }

            let function_count = reader.read_count("function providers", MAXIMUM_PROVIDERS)?;
            for _ in 0..function_count {
                let index = reader.read_u16("function provider index")?;
                let target_count = reader.read_count("function targets", MAXIMUM_TARGETS)?;
                if target_count == 0 {
                    return Err(protocol(format!("function provider {index} has no targets")));
                }

                let mut targets = Vec::with_capacity(target_count);
                for _ in 0..target_count {
                    let kind = reader.read_u8("function target kind")?;
                    let mut value = non_empty(reader.read_bytes("function target")?.to_vec(), "function target")?;
                    targets.push(match kind {
                        TARGET_EXACT => FunctionTarget::Exact(value),
                        TARGET_PREFIX => FunctionTarget::Prefix(value),
                        TARGET_NAMESPACE => {
                            if value.last() != Some(&b'\\') {
                                value.push(b'\\');
                            }
                            FunctionTarget::Namespace(value)
                        }
                        _ => return Err(protocol(format!("function provider {index} has unknown target kind {kind}"))),
                    });
                }

                function_providers.push(FunctionProvider { plugin: identifier.clone(), index, targets });
            }

            let method_count = reader.read_count("method providers", MAXIMUM_PROVIDERS)?;
            for _ in 0..method_count {
                let index = reader.read_u16("method provider index")?;
                let target_count = reader.read_count("method targets", MAXIMUM_TARGETS)?;
                if target_count == 0 {
                    return Err(protocol(format!("method provider {index} has no targets")));
                }

                let mut targets = Vec::with_capacity(target_count);
                for _ in 0..target_count {
                    let class = non_empty(reader.read_bytes("method target class")?.to_vec(), "method target class")?;
                    let method =
                        non_empty(reader.read_bytes("method target method")?.to_vec(), "method target method")?;
                    validate_method_pattern(&class)?;
                    validate_method_pattern(&method)?;
                    targets.push(MethodTarget { class, method });
                }

                method_providers.push(MethodProvider { plugin: identifier.clone(), index, targets });
            }

            let plugin = ExternalPlugin {
                index,
                extension: extension_identifier.clone(),
                identifier,
                name,
                description,
                aliases,
                default_enabled,
                initialization: lifecycle & 8 != 0,
                before_analysis: lifecycle & 1 != 0,
                after_file_analysis: lifecycle & 2 != 0,
                after_analysis: lifecycle & 4 != 0,
            };

            extension_plugins.push(plugin.clone());
            plugins.push(plugin);
        }

        extensions.push(ExternalExtension {
            identifier: extension_identifier,
            name: extension_name,
            version: extension_version,
            plugins: extension_plugins,
        });
    }

    reader.finish()?;
    validate_provider_indices(
        function_providers.iter().map(|provider| provider.index),
        "function return-type provider",
    )?;

    validate_provider_indices(method_providers.iter().map(|provider| provider.index), "method return-type provider")?;
    Ok(Registration {
        extensions,
        plugins,
        function_providers,
        method_providers,
        initialization_plugins,
        before_analysis_plugins,
        after_file_analysis_plugins,
        after_analysis_plugins,
    })
}

pub(super) fn encode_function_return_type_request<'type_info>(
    provider_indices: &[u16],
    name: &[u8],
    invocation: &Invocation<'_, '_, '_>,
    artifacts: &'type_info AnalysisArtifacts,
    source_file: &File,
    generation: u64,
    trace_enabled: bool,
) -> Result<ReturnTypeRequest<'type_info>, ExternalAnalyzerError> {
    encode_return_type_request(
        provider_indices,
        INVOCATION_FUNCTION,
        None,
        name,
        invocation,
        artifacts,
        source_file,
        generation,
        trace_enabled,
    )
}

pub(super) fn encode_method_return_type_request<'type_info>(
    provider_indices: &[u16],
    class: &[u8],
    method: &[u8],
    invocation: &Invocation<'_, '_, '_>,
    artifacts: &'type_info AnalysisArtifacts,
    source_file: &File,
    generation: u64,
    trace_enabled: bool,
) -> Result<ReturnTypeRequest<'type_info>, ExternalAnalyzerError> {
    encode_return_type_request(
        provider_indices,
        INVOCATION_METHOD,
        Some(class),
        method,
        invocation,
        artifacts,
        source_file,
        generation,
        trace_enabled,
    )
}

fn encode_return_type_request<'type_info>(
    provider_indices: &[u16],
    invocation_kind: u8,
    class: Option<&[u8]>,
    name: &[u8],
    invocation: &Invocation<'_, '_, '_>,
    artifacts: &'type_info AnalysisArtifacts,
    source_file: &File,
    generation: u64,
    trace_enabled: bool,
) -> Result<ReturnTypeRequest<'type_info>, ExternalAnalyzerError> {
    let mut writer = message_writer(RETURN_TYPE_REQUEST);
    let mut types = Vec::new();
    let mut typed_arguments = 0usize;
    let mut type_snapshot_duration = Duration::ZERO;
    writer.write_u64(generation);
    writer.write_u8(invocation_kind);
    writer.write_u16(
        u16::try_from(provider_indices.len()).map_err(|_| protocol("more than u16::MAX providers matched"))?,
    );

    for index in provider_indices {
        writer.write_u16(*index);
    }

    if let Some(class) = class {
        writer.write_bytes(class)?;
    }

    writer.write_bytes(name)?;
    writer.write_u32(invocation.span.start.offset);
    writer.write_u32(invocation.span.end.offset);

    let argument_count = invocation.arguments_source.argument_count();
    writer
        .write_u16(u16::try_from(argument_count).map_err(|_| protocol("invocation has more than u16::MAX arguments"))?);
    for argument in invocation.arguments_source.iter_arguments() {
        writer.write_optional_string(argument.get_parameter_name().map(String::from_utf8_lossy).as_deref())?;
        writer.write_bool(argument.is_unpacked());
        writer.write_bool(argument.is_placeholder());
        let span = argument.span();
        writer.write_u32(span.start.offset);
        writer.write_u32(span.end.offset);
        let value = argument.value();
        let expression = value
            .map(HasSpan::span)
            .and_then(|span| source_file.contents.get(span.start.offset as usize..span.end.offset as usize))
            .unwrap_or_default();

        writer.write_bytes(expression)?;
        let argument_type = value.and_then(|value| artifacts.get_expression_type(value));
        writer.write_bool(argument_type.is_some());
        if let Some(argument_type) = argument_type {
            typed_arguments += 1;
            writer.write_bytes(argument_type.get_id().as_bytes())?;
            let snapshot_start = trace_enabled.then(Instant::now);
            encode_union_snapshot(&mut writer, argument_type, &mut types, 0)?;
            if let Some(start) = snapshot_start {
                type_snapshot_duration = type_snapshot_duration.saturating_add(start.elapsed());
            }
        }
    }

    Ok(ReturnTypeRequest {
        payload: writer.finish(),
        types,
        arguments: argument_count,
        typed_arguments,
        type_snapshot_duration,
    })
}

pub(super) fn encode_union_snapshot<'type_info>(
    writer: &mut PayloadWriter,
    union: &'type_info TUnion,
    types: &mut Vec<&'type_info TUnion>,
    depth: usize,
) -> Result<(), ExternalAnalyzerError> {
    ensure_type_depth(depth)?;
    let handle = u32::try_from(types.len()).map_err(|_| protocol("external type handle exceeds u32::MAX"))?;
    types.push(union);
    writer.write_u32(handle);
    let mut flags = 0u16;
    flags |= u16::from(union.had_template());
    flags |= u16::from(union.by_reference()) << 1;
    flags |= u16::from(union.reference_free()) << 2;
    flags |= u16::from(union.possibly_undefined_from_try()) << 3;
    flags |= u16::from(union.possibly_undefined()) << 4;
    flags |= u16::from(union.ignore_nullable_issues()) << 5;
    flags |= u16::from(union.ignore_falsable_issues()) << 6;
    flags |= u16::from(union.from_template_default()) << 7;
    flags |= u16::from(union.populated()) << 8;
    flags |= u16::from(union.has_nullsafe_null()) << 9;
    flags |= u16::from(union.from_unspecified_template()) << 10;
    writer.write_u16(flags);
    write_snapshot_count(writer, union.types.len(), "union atomic types")?;
    for atomic in union.types.as_ref() {
        encode_atomic_snapshot(writer, atomic, types, depth + 1)?;
    }

    Ok(())
}

#[allow(clippy::too_many_lines)]
fn encode_atomic_snapshot<'type_info>(
    writer: &mut PayloadWriter,
    atomic: &'type_info TAtomic,
    types: &mut Vec<&'type_info TUnion>,
    depth: usize,
) -> Result<(), ExternalAnalyzerError> {
    ensure_type_depth(depth)?;
    match atomic {
        TAtomic::Scalar(scalar) => {
            writer.write_u8(SNAPSHOT_SCALAR);
            encode_scalar_snapshot(writer, scalar, types, depth + 1)?;
        }
        TAtomic::Callable(callable) => {
            writer.write_u8(SNAPSHOT_CALLABLE);
            encode_callable_snapshot(writer, callable, types, depth + 1)?;
        }
        TAtomic::Mixed(mixed) => {
            writer.write_u8(SNAPSHOT_MIXED);
            let mut flags = 0u8;
            flags |= u8::from(mixed.is_isset_from_loop());
            flags |= u8::from(mixed.is_non_null()) << 1;
            flags |= u8::from(mixed.is_empty()) << 2;
            writer.write_u8(flags);
            writer.write_u8(match mixed.get_truthiness() {
                TMixedTruthiness::Undetermined => 0,
                TMixedTruthiness::Truthy => 1,
                TMixedTruthiness::Falsy => 2,
            });
        }
        TAtomic::Object(object) => {
            writer.write_u8(SNAPSHOT_OBJECT);
            encode_object_snapshot(writer, object, types, depth + 1)?;
        }
        TAtomic::Array(array) => {
            writer.write_u8(SNAPSHOT_ARRAY);
            encode_array_snapshot(writer, array, types, depth + 1)?;
        }
        TAtomic::Iterable(iterable) => {
            writer.write_u8(SNAPSHOT_ITERABLE);
            encode_union_snapshot(writer, &iterable.key_type, types, depth + 1)?;
            encode_union_snapshot(writer, &iterable.value_type, types, depth + 1)?;
            encode_optional_atomic_snapshots(writer, iterable.intersection_types.as_deref(), types, depth + 1)?;
        }
        TAtomic::Resource(resource) => {
            writer.write_u8(SNAPSHOT_RESOURCE);
            writer.write_u8(match resource.closed {
                None => 0,
                Some(false) => 1,
                Some(true) => 2,
            });
        }
        TAtomic::Reference(reference) => {
            writer.write_u8(SNAPSHOT_REFERENCE);
            encode_reference_snapshot(writer, reference, types, depth + 1)?;
        }
        TAtomic::GenericParameter(parameter) => {
            writer.write_u8(SNAPSHOT_GENERIC_PARAMETER);
            writer.write_bytes(parameter.parameter_name.as_bytes())?;
            encode_union_snapshot(writer, &parameter.constraint, types, depth + 1)?;
            encode_generic_parent(writer, parameter.defining_entity)?;
            encode_optional_atomic_snapshots(writer, parameter.intersection_types.as_deref(), types, depth + 1)?;
        }
        TAtomic::Variable(variable) => {
            writer.write_u8(SNAPSHOT_VARIABLE);
            writer.write_bytes(variable.as_bytes())?;
        }
        TAtomic::Conditional(conditional) => {
            writer.write_u8(SNAPSHOT_CONDITIONAL);
            encode_union_snapshot(writer, &conditional.subject, types, depth + 1)?;
            encode_union_snapshot(writer, &conditional.target, types, depth + 1)?;
            encode_union_snapshot(writer, &conditional.then, types, depth + 1)?;
            encode_union_snapshot(writer, &conditional.otherwise, types, depth + 1)?;
            writer.write_bool(conditional.negated);
        }
        TAtomic::Derived(derived) => {
            writer.write_u8(SNAPSHOT_DERIVED);
            encode_derived_snapshot(writer, derived, types, depth + 1)?;
        }
        TAtomic::Alias(alias) => {
            writer.write_u8(SNAPSHOT_ALIAS);
            writer.write_bytes(alias.get_class_name().as_bytes())?;
            writer.write_bytes(alias.get_alias_name().as_bytes())?;
        }
        TAtomic::Never => writer.write_u8(SNAPSHOT_NEVER),
        TAtomic::Null => writer.write_u8(SNAPSHOT_NULL),
        TAtomic::Void => writer.write_u8(SNAPSHOT_VOID),
        TAtomic::Placeholder => writer.write_u8(SNAPSHOT_PLACEHOLDER),
    }

    Ok(())
}

fn encode_scalar_snapshot<'type_info>(
    writer: &mut PayloadWriter,
    scalar: &'type_info TScalar,
    types: &mut Vec<&'type_info TUnion>,
    depth: usize,
) -> Result<(), ExternalAnalyzerError> {
    match scalar {
        TScalar::Generic => writer.write_u8(1),
        TScalar::Numeric => writer.write_u8(2),
        TScalar::ArrayKey => writer.write_u8(3),
        TScalar::Bool(boolean) => {
            writer.write_u8(4);
            writer.write_u8(match boolean.value {
                None => 0,
                Some(false) => 1,
                Some(true) => 2,
            });
        }
        TScalar::Integer(integer) => {
            writer.write_u8(5);
            match integer {
                TInteger::Literal(value) => {
                    writer.write_u8(1);
                    writer.write_u64(*value as u64);
                }
                TInteger::From(value) => {
                    writer.write_u8(2);
                    writer.write_u64(*value as u64);
                }
                TInteger::To(value) => {
                    writer.write_u8(3);
                    writer.write_u64(*value as u64);
                }
                TInteger::Range(minimum, maximum) => {
                    writer.write_u8(4);
                    writer.write_u64(*minimum as u64);
                    writer.write_u64(*maximum as u64);
                }
                TInteger::Unspecified => writer.write_u8(5),
                TInteger::UnspecifiedLiteral => writer.write_u8(6),
            }
        }
        TScalar::Float(float) => {
            writer.write_u8(6);
            match float {
                TFloat::Float => writer.write_u8(1),
                TFloat::UnspecifiedLiteral => writer.write_u8(2),
                TFloat::Literal(value) => {
                    writer.write_u8(3);
                    writer.write_u64(value.into_inner().to_bits());
                }
            }
        }
        TScalar::String(string) => {
            writer.write_u8(7);
            match string.literal {
                None => writer.write_u8(0),
                Some(TStringLiteral::Unspecified) => writer.write_u8(1),
                Some(TStringLiteral::Value(value)) => {
                    writer.write_u8(2);
                    writer.write_bytes(value.as_bytes())?;
                }
            }

            let mut flags = 0u8;
            flags |= u8::from(string.is_numeric);
            flags |= u8::from(string.is_truthy) << 1;
            flags |= u8::from(string.is_non_empty) << 2;
            flags |= u8::from(string.is_callable) << 3;
            writer.write_u8(flags);
            writer.write_u8(match string.casing {
                TStringCasing::Unspecified => 0,
                TStringCasing::Lowercase => 1,
                TStringCasing::Uppercase => 2,
            });
        }
        TScalar::ClassLikeString(class_string) => {
            writer.write_u8(8);
            match class_string {
                TClassLikeString::Any { kind } => {
                    writer.write_u8(1);
                    encode_class_like_string_kind(writer, *kind);
                }
                TClassLikeString::Generic { kind, parameter_name, defining_entity, constraint } => {
                    writer.write_u8(2);
                    encode_class_like_string_kind(writer, *kind);
                    writer.write_bytes(parameter_name.as_bytes())?;
                    encode_generic_parent(writer, *defining_entity)?;
                    encode_atomic_snapshot(writer, constraint, types, depth + 1)?;
                }
                TClassLikeString::Literal { value } => {
                    writer.write_u8(3);
                    writer.write_bytes(value.as_bytes())?;
                }
                TClassLikeString::OfType { kind, constraint } => {
                    writer.write_u8(4);
                    encode_class_like_string_kind(writer, *kind);
                    encode_atomic_snapshot(writer, constraint, types, depth + 1)?;
                }
            }
        }
    }

    Ok(())
}

fn encode_callable_snapshot<'type_info>(
    writer: &mut PayloadWriter,
    callable: &'type_info TCallable,
    types: &mut Vec<&'type_info TUnion>,
    depth: usize,
) -> Result<(), ExternalAnalyzerError> {
    match callable {
        TCallable::Signature(signature) => {
            writer.write_u8(1);
            writer.write_bool(signature.is_pure);
            writer.write_bool(signature.is_closure);
            write_snapshot_count(writer, signature.parameters.len(), "callable parameters")?;
            for parameter in &signature.parameters {
                writer.write_bool(parameter.get_name().is_some());
                if let Some(name) = parameter.get_name() {
                    writer.write_bytes(name.0.as_bytes())?;
                }

                writer.write_bool(parameter.get_type_signature().is_some());
                if let Some(parameter_type) = parameter.get_type_signature() {
                    encode_union_snapshot(writer, parameter_type, types, depth + 1)?;
                }

                writer.write_bool(parameter.is_by_reference());
                writer.write_bool(parameter.is_variadic());
                writer.write_bool(parameter.has_default());
            }

            writer.write_bool(signature.return_type.is_some());
            if let Some(return_type) = signature.return_type.as_deref() {
                encode_union_snapshot(writer, return_type, types, depth + 1)?;
            }

            writer.write_bool(signature.source.is_some());
            if let Some(source) = signature.source {
                encode_function_like_identifier(writer, source)?;
            }

            write_snapshot_count(writer, signature.constraints.len(), "callable constraints")?;
            for constraint in &signature.constraints {
                write_snapshot_count(writer, constraint.parameter_names.len(), "callable constraint names")?;
                for name in &constraint.parameter_names {
                    writer.write_bytes(name.0.as_bytes())?;
                }

                encode_union_snapshot(writer, &constraint.input_type, types, depth + 1)?;
                encode_union_snapshot(writer, &constraint.parameter_type, types, depth + 1)?;
            }
        }
        TCallable::Alias(identifier) => {
            writer.write_u8(2);
            encode_function_like_identifier(writer, *identifier)?;
        }
    }

    Ok(())
}

#[allow(clippy::too_many_lines)]
fn encode_object_snapshot<'type_info>(
    writer: &mut PayloadWriter,
    object: &'type_info TObject,
    types: &mut Vec<&'type_info TUnion>,
    depth: usize,
) -> Result<(), ExternalAnalyzerError> {
    match object {
        TObject::Any => writer.write_u8(1),
        TObject::Named(named) => {
            writer.write_u8(2);
            writer.write_bytes(named.name.as_bytes())?;
            encode_optional_unions(writer, named.type_parameters.as_deref(), types, depth + 1)?;
            encode_optional_variances(writer, named.variances.as_deref())?;
            writer.write_bool(named.is_static);
            writer.write_bool(named.is_this);
            encode_optional_atomic_snapshots(writer, named.intersection_types.as_deref(), types, depth + 1)?;
            writer.write_bool(named.remapped_parameters);
        }
        TObject::Enum(r#enum) => {
            writer.write_u8(3);
            writer.write_bytes(r#enum.name.as_bytes())?;
            writer.write_optional_string(r#enum.case.map(|case| case.to_string()).as_deref())?;
        }
        TObject::WithProperties(shape) => {
            writer.write_u8(4);
            writer.write_bool(shape.sealed);
            write_snapshot_count(writer, shape.known_properties.len(), "object properties")?;
            for (name, (optional, property_type)) in &shape.known_properties {
                writer.write_bytes(name.as_bytes())?;
                writer.write_bool(*optional);
                encode_union_snapshot(writer, property_type, types, depth + 1)?;
            }
        }
        TObject::HasMethod(method) => {
            writer.write_u8(5);
            writer.write_bytes(method.method.as_bytes())?;
            encode_optional_atomic_snapshots(writer, method.intersection_types.as_deref(), types, depth + 1)?;
        }
        TObject::HasProperty(property) => {
            writer.write_u8(6);
            writer.write_bytes(property.property.as_bytes())?;
            encode_optional_atomic_snapshots(writer, property.intersection_types.as_deref(), types, depth + 1)?;
        }
    }

    Ok(())
}

fn encode_array_snapshot<'type_info>(
    writer: &mut PayloadWriter,
    array: &'type_info TArray,
    types: &mut Vec<&'type_info TUnion>,
    depth: usize,
) -> Result<(), ExternalAnalyzerError> {
    match array {
        TArray::List(list) => {
            writer.write_u8(1);
            encode_union_snapshot(writer, &list.element_type, types, depth + 1)?;
            writer.write_bool(list.known_elements.is_some());
            if let Some(elements) = &list.known_elements {
                write_snapshot_count(writer, elements.len(), "known list elements")?;
                for (index, (optional, element_type)) in elements {
                    writer.write_u64(*index as u64);
                    writer.write_bool(*optional);
                    encode_union_snapshot(writer, element_type, types, depth + 1)?;
                }
            }

            writer.write_bool(list.known_count.is_some());
            if let Some(count) = list.known_count {
                writer.write_u64(count as u64);
            }

            writer.write_bool(list.non_empty);
        }
        TArray::Keyed(keyed) => {
            writer.write_u8(2);
            writer.write_bool(keyed.known_items.is_some());
            if let Some(items) = &keyed.known_items {
                write_snapshot_count(writer, items.len(), "known array items")?;
                for (key, (optional, value_type)) in items {
                    encode_array_key(writer, key)?;
                    writer.write_bool(*optional);
                    encode_union_snapshot(writer, value_type, types, depth + 1)?;
                }
            }

            writer.write_bool(keyed.parameters.is_some());
            if let Some((key_type, value_type)) = &keyed.parameters {
                encode_union_snapshot(writer, key_type, types, depth + 1)?;
                encode_union_snapshot(writer, value_type, types, depth + 1)?;
            }

            writer.write_bool(keyed.non_empty);
        }
    }

    Ok(())
}

fn encode_reference_snapshot<'type_info>(
    writer: &mut PayloadWriter,
    reference: &'type_info TReference,
    types: &mut Vec<&'type_info TUnion>,
    depth: usize,
) -> Result<(), ExternalAnalyzerError> {
    match reference {
        TReference::Symbol { name, parameters, variances, intersection_types } => {
            writer.write_u8(1);
            writer.write_bytes(name.as_bytes())?;
            encode_optional_unions(writer, parameters.as_deref(), types, depth + 1)?;
            encode_optional_variances(writer, variances.as_deref())?;
            encode_optional_atomic_snapshots(writer, intersection_types.as_deref(), types, depth + 1)?;
        }
        TReference::Member { class_like_name, member_selector } => {
            writer.write_u8(2);
            writer.write_bytes(class_like_name.as_bytes())?;
            match member_selector {
                TReferenceMemberSelector::Wildcard => writer.write_u8(1),
                TReferenceMemberSelector::Identifier(name) => {
                    writer.write_u8(2);
                    writer.write_bytes(name.as_bytes())?;
                }
                TReferenceMemberSelector::StartsWith(prefix) => {
                    writer.write_u8(3);
                    writer.write_bytes(prefix.as_bytes())?;
                }
                TReferenceMemberSelector::EndsWith(suffix) => {
                    writer.write_u8(4);
                    writer.write_bytes(suffix.as_bytes())?;
                }
            }
        }
        TReference::Global { selector } => {
            writer.write_u8(3);
            match selector {
                TGlobalReferenceSelector::StartsWith(prefix) => {
                    writer.write_u8(3);
                    writer.write_bytes(prefix.as_bytes())?;
                }
                TGlobalReferenceSelector::EndsWith(suffix) => {
                    writer.write_u8(4);
                    writer.write_bytes(suffix.as_bytes())?;
                }
            }
        }
    }

    Ok(())
}

fn encode_derived_snapshot<'type_info>(
    writer: &mut PayloadWriter,
    derived: &'type_info TDerived,
    types: &mut Vec<&'type_info TUnion>,
    depth: usize,
) -> Result<(), ExternalAnalyzerError> {
    match derived {
        TDerived::KeyOf(value) => {
            writer.write_u8(1);
            encode_union_snapshot(writer, value.get_target_type(), types, depth + 1)?;
        }
        TDerived::ValueOf(value) => {
            writer.write_u8(2);
            encode_union_snapshot(writer, value.get_target_type(), types, depth + 1)?;
        }
        TDerived::IntMask(mask) => {
            writer.write_u8(3);
            write_snapshot_count(writer, mask.get_values().len(), "int-mask values")?;
            for value in mask.get_values() {
                encode_union_snapshot(writer, value, types, depth + 1)?;
            }
        }
        TDerived::IntMaskOf(value) => {
            writer.write_u8(4);
            encode_union_snapshot(writer, value.get_target_type(), types, depth + 1)?;
        }
        TDerived::PropertiesOf(properties) => {
            writer.write_u8(5);
            writer.write_u8(match properties.visibility() {
                None => 0,
                Some(Visibility::Public) => 1,
                Some(Visibility::Protected) => 2,
                Some(Visibility::Private) => 3,
            });

            encode_union_snapshot(writer, properties.get_target_type(), types, depth + 1)?;
        }
        TDerived::IndexAccess(access) => {
            writer.write_u8(6);
            encode_union_snapshot(writer, access.get_target_type(), types, depth + 1)?;
            encode_union_snapshot(writer, access.get_index_type(), types, depth + 1)?;
        }
        TDerived::New(new_type) => {
            writer.write_u8(7);
            encode_union_snapshot(writer, new_type.get_target_type(), types, depth + 1)?;
        }
        TDerived::TemplateType(template) => {
            writer.write_u8(8);
            encode_union_snapshot(writer, template.get_object(), types, depth + 1)?;
            encode_union_snapshot(writer, template.get_class_name(), types, depth + 1)?;
            encode_union_snapshot(writer, template.get_template_name(), types, depth + 1)?;
        }
        TDerived::Intersection(intersection) => {
            writer.write_u8(9);
            encode_union_snapshot(writer, intersection.get_base_type(), types, depth + 1)?;
            let intersections = intersection.get_intersection_types().unwrap_or_default();
            write_snapshot_count(writer, intersections.len(), "derived intersections")?;
            for atomic in intersections {
                encode_atomic_snapshot(writer, atomic, types, depth + 1)?;
            }
        }
    }

    Ok(())
}

fn encode_optional_unions<'type_info>(
    writer: &mut PayloadWriter,
    unions: Option<&'type_info [TUnion]>,
    types: &mut Vec<&'type_info TUnion>,
    depth: usize,
) -> Result<(), ExternalAnalyzerError> {
    writer.write_bool(unions.is_some());
    if let Some(unions) = unions {
        write_snapshot_count(writer, unions.len(), "nested union types")?;
        for union in unions {
            encode_union_snapshot(writer, union, types, depth + 1)?;
        }
    }
    Ok(())
}

fn encode_optional_atomic_snapshots<'type_info>(
    writer: &mut PayloadWriter,
    atomics: Option<&'type_info [TAtomic]>,
    types: &mut Vec<&'type_info TUnion>,
    depth: usize,
) -> Result<(), ExternalAnalyzerError> {
    writer.write_bool(atomics.is_some());
    if let Some(atomics) = atomics {
        write_snapshot_count(writer, atomics.len(), "intersection atomic types")?;
        for atomic in atomics {
            encode_atomic_snapshot(writer, atomic, types, depth + 1)?;
        }
    }

    Ok(())
}

fn encode_optional_variances(
    writer: &mut PayloadWriter,
    variances: Option<&[Variance]>,
) -> Result<(), ExternalAnalyzerError> {
    writer.write_bool(variances.is_some());
    if let Some(variances) = variances {
        write_snapshot_count(writer, variances.len(), "type variances")?;
        for variance in variances {
            writer.write_u8(match variance {
                Variance::Invariant => 1,
                Variance::Covariant => 2,
                Variance::Contravariant => 3,
                Variance::Bivariant => 4,
            });
        }
    }

    Ok(())
}

pub(super) fn encode_generic_parent(
    writer: &mut PayloadWriter,
    parent: GenericParent,
) -> Result<(), ExternalAnalyzerError> {
    match parent {
        GenericParent::ClassLike(name) => {
            writer.write_u8(1);
            writer.write_bytes(name.as_bytes())?;
        }
        GenericParent::FunctionLike((name, member)) => {
            writer.write_u8(2);
            writer.write_bytes(name.as_bytes())?;
            writer.write_bytes(member.as_bytes())?;
        }
    }

    Ok(())
}

fn encode_function_like_identifier(
    writer: &mut PayloadWriter,
    identifier: FunctionLikeIdentifier,
) -> Result<(), ExternalAnalyzerError> {
    match identifier {
        FunctionLikeIdentifier::Function(name) => {
            writer.write_u8(1);
            writer.write_bytes(name.as_bytes())?;
        }
        FunctionLikeIdentifier::Method(class, method) => {
            writer.write_u8(2);
            writer.write_bytes(class.as_bytes())?;
            writer.write_bytes(method.as_bytes())?;
        }
        FunctionLikeIdentifier::Closure(name) => {
            writer.write_u8(3);
            writer.write_bytes(name.as_bytes())?;
        }
    }

    Ok(())
}

fn encode_class_like_string_kind(writer: &mut PayloadWriter, kind: TClassLikeStringKind) {
    writer.write_u8(match kind {
        TClassLikeStringKind::Class => 1,
        TClassLikeStringKind::Interface => 2,
        TClassLikeStringKind::Enum => 3,
        TClassLikeStringKind::Trait => 4,
    });
}

fn encode_array_key(writer: &mut PayloadWriter, key: &ArrayKey) -> Result<(), ExternalAnalyzerError> {
    match key {
        ArrayKey::Integer(value) => {
            writer.write_u8(1);
            writer.write_u64(*value as u64);
        }
        ArrayKey::String(value) => {
            writer.write_u8(2);
            writer.write_bytes(value.as_bytes())?;
        }
        ArrayKey::ClassLikeConstant { class_like_name, constant_name } => {
            writer.write_u8(3);
            writer.write_bytes(class_like_name.as_bytes())?;
            writer.write_bytes(constant_name.as_bytes())?;
        }
    }

    Ok(())
}

fn write_snapshot_count(
    writer: &mut PayloadWriter,
    count: usize,
    field: &'static str,
) -> Result<(), ExternalAnalyzerError> {
    if count > MAXIMUM_TYPE_MEMBERS {
        return Err(protocol(format!("{field} exceeds the maximum member count")));
    }

    writer.write_u32(u32::try_from(count).map_err(|_| protocol(format!("{field} exceeds u32::MAX")))?);
    Ok(())
}

fn ensure_type_depth(depth: usize) -> Result<(), ExternalAnalyzerError> {
    if depth >= MAXIMUM_TYPE_DEPTH {
        Err(protocol("external type snapshot exceeds the maximum nesting depth"))
    } else {
        Ok(())
    }
}

pub(super) fn decode_return_type_response<'type_info, F>(
    payload: &[u8],
    argument_type: F,
) -> Result<Option<TUnion>, ExternalAnalyzerError>
where
    F: Fn(usize) -> Option<&'type_info TUnion>,
{
    let mut reader = message_reader(payload, RETURN_TYPE_RESPONSE)?;
    let result =
        if reader.read_bool("handled flag")? { Some(decode_type(&mut reader, &argument_type, 0)?) } else { None };
    reader.finish()?;
    Ok(result)
}

pub(super) fn handle_type_comparison_request<'type_info, F>(
    payload: &[u8],
    codebase: &CodebaseMetadata,
    argument_type: F,
) -> Result<Vec<u8>, ExternalAnalyzerError>
where
    F: Fn(usize) -> Option<&'type_info TUnion>,
{
    let mut reader = message_reader(payload, TYPE_COMPARISON_REQUEST)?;
    let operation = reader.read_u8("type comparison operation")?;
    let left = decode_type(&mut reader, &argument_type, 0)?;
    let right = decode_type(&mut reader, &argument_type, 0)?;
    reader.finish()?;

    let result = match operation {
        TYPE_COMPARISON_EQUAL => left == right,
        TYPE_COMPARISON_CONTAINED_BY => union_comparator::is_contained_by(
            codebase,
            &left,
            &right,
            false,
            false,
            false,
            &mut ComparisonResult::default(),
        ),
        TYPE_COMPARISON_CAN_BE_IDENTICAL => {
            union_comparator::can_expression_types_be_identical(codebase, &left, &right, false, false)
        }
        unknown => return Err(protocol(format!("unknown type comparison operation {unknown}"))),
    };

    let mut writer = message_writer(TYPE_COMPARISON_RESPONSE);
    writer.write_bool(result);
    Ok(writer.finish())
}

pub(super) fn handle_nested_request<'type_info, F>(
    payload: &[u8],
    codebase: &CodebaseMetadata,
    session: &ExternalAnalysisSession,
    argument_type: F,
) -> Result<(NestedRequestKind, Vec<u8>), ExternalAnalyzerError>
where
    F: Fn(usize) -> Option<&'type_info TUnion>,
{
    let kind = message_kind(payload)?;
    match kind {
        TYPE_COMPARISON_REQUEST => handle_type_comparison_request(payload, codebase, argument_type)
            .map(|response| (NestedRequestKind::TypeComparison, response)),
        metadata::CODEBASE_QUERY_REQUEST => {
            let mut reader = message_reader(payload, metadata::CODEBASE_QUERY_REQUEST)?;
            let mut writer = message_writer(metadata::CODEBASE_QUERY_RESPONSE);
            metadata::handle_query(&mut reader, codebase, session, &mut writer)?;
            reader.finish()?;
            Ok((NestedRequestKind::CodebaseQuery, writer.finish()))
        }
        unknown => Err(protocol(format!("unknown nested analyzer request kind {unknown}"))),
    }
}

fn decode_type<'type_info, F>(
    reader: &mut PayloadReader<'_>,
    argument_type: &F,
    depth: usize,
) -> Result<TUnion, ExternalAnalyzerError>
where
    F: Fn(usize) -> Option<&'type_info TUnion>,
{
    if depth >= MAXIMUM_TYPE_DEPTH {
        return Err(protocol("external type exceeds the maximum nesting depth"));
    }

    let tag = reader.read_u8("type tag")?;
    let next_depth = depth + 1;
    Ok(match tag {
        TYPE_REFERENCE => {
            let handle = reader.read_u32("type reference handle")? as usize;
            argument_type(handle)
                .cloned()
                .ok_or_else(|| protocol(format!("type references unknown request-local handle {handle}")))?
        }
        TYPE_MIXED => get_mixed(),
        TYPE_NEVER => get_never(),
        TYPE_NULL => TUnion::from_atomic(TAtomic::Null),
        TYPE_VOID => TUnion::from_atomic(TAtomic::Void),
        TYPE_BOOL => get_bool(),
        TYPE_TRUE => get_true(),
        TYPE_FALSE => get_false(),
        TYPE_INT => get_int(),
        TYPE_FLOAT => get_float(),
        TYPE_STRING => get_string(),
        TYPE_LITERAL_STRING => get_literal_string(word(reader.read_bytes("literal string")?)),
        TYPE_OBJECT => get_object(),
        TYPE_NAMED_OBJECT => {
            let name = reader.read_bytes("object name")?;
            if name.is_empty() {
                return Err(protocol("named object type has an empty name"));
            }

            let parameter_count = reader.read_count("object type parameters", MAXIMUM_TYPE_MEMBERS)?;
            let parameters = if parameter_count == 0 {
                None
            } else {
                let mut parameters = Vec::with_capacity(parameter_count);
                for _ in 0..parameter_count {
                    parameters.push(decode_type(reader, argument_type, next_depth)?);
                }
                Some(parameters)
            };

            TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new_with_type_parameters(
                word(name),
                parameters,
            ))))
        }
        TYPE_ARRAY => {
            let key = decode_type(reader, argument_type, next_depth)?;
            let value = decode_type(reader, argument_type, next_depth)?;
            get_keyed_array(key, value)
        }
        TYPE_LIST => get_list(decode_type(reader, argument_type, next_depth)?),
        TYPE_UNION => {
            let member_count = reader.read_count("union members", MAXIMUM_TYPE_MEMBERS)?;
            if member_count == 0 {
                return Err(protocol("union type contains no members"));
            }

            let mut atomics = Vec::with_capacity(member_count);
            for _ in 0..member_count {
                let member = decode_type(reader, argument_type, next_depth)?;
                atomics.extend(member.types.into_owned());
            }

            TUnion::from_vec(atomics)
        }
        TYPE_NON_NEGATIVE_INT => get_non_negative_int(),
        TYPE_NON_EMPTY_STRING => get_non_empty_string(),
        TYPE_LITERAL_INT => TUnion::from_atomic(TAtomic::Scalar(TScalar::Integer(TInteger::Literal(
            reader.read_u64("literal integer")? as i64,
        )))),
        TYPE_COMPLETE => decode_complete_union(reader, next_depth)?,
        unknown => return Err(protocol(format!("unknown external type tag {unknown}"))),
    })
}

pub(super) fn decode_complete_union(
    reader: &mut PayloadReader<'_>,
    depth: usize,
) -> Result<TUnion, ExternalAnalyzerError> {
    ensure_type_depth(depth)?;
    let flags = reader.read_u16("complete union flags")?;
    let count = reader.read_count("complete union atomic types", MAXIMUM_TYPE_MEMBERS)?;
    if count == 0 {
        return Err(protocol("complete union type contains no atomic types"));
    }

    let mut atomics = Vec::with_capacity(count);
    for _ in 0..count {
        atomics.push(decode_complete_atomic(reader, depth + 1)?);
    }

    let mut union = TUnion::from_vec(atomics);
    union.set_had_template(flags & (1 << 0) != 0);
    union.set_by_reference(flags & (1 << 1) != 0);
    union.set_reference_free(flags & (1 << 2) != 0);
    union.set_possibly_undefined_from_try(flags & (1 << 3) != 0);
    union.set_possibly_undefined(flags & (1 << 4) != 0, None);
    union.set_ignore_nullable_issues(flags & (1 << 5) != 0);
    union.set_ignore_falsable_issues(flags & (1 << 6) != 0);
    union.set_from_template_default(flags & (1 << 7) != 0);
    union.set_populated(flags & (1 << 8) != 0);
    union.set_nullsafe_null(flags & (1 << 9) != 0);
    union.set_from_unspecified_template(flags & (1 << 10) != 0);
    Ok(union)
}

#[allow(clippy::too_many_lines)]
fn decode_complete_atomic(reader: &mut PayloadReader<'_>, depth: usize) -> Result<TAtomic, ExternalAnalyzerError> {
    ensure_type_depth(depth)?;
    Ok(match reader.read_u8("complete atomic type tag")? {
        SNAPSHOT_SCALAR => TAtomic::Scalar(decode_complete_scalar(reader, depth + 1)?),
        SNAPSHOT_CALLABLE => TAtomic::Callable(decode_complete_callable(reader, depth + 1)?),
        SNAPSHOT_MIXED => {
            let flags = reader.read_u8("mixed flags")?;
            let truthiness = match reader.read_u8("mixed truthiness")? {
                0 => TMixedTruthiness::Undetermined,
                1 => TMixedTruthiness::Truthy,
                2 => TMixedTruthiness::Falsy,
                unknown => return Err(protocol(format!("unknown mixed truthiness {unknown}"))),
            };

            let mut mixed = TMixed::new()
                .with_is_isset_from_loop(flags & (1 << 0) != 0)
                .with_is_non_null(flags & (1 << 1) != 0)
                .with_truthiness(truthiness);

            if flags & (1 << 2) != 0 {
                mixed = mixed.as_empty();
            }

            TAtomic::Mixed(mixed)
        }
        SNAPSHOT_OBJECT => TAtomic::Object(decode_complete_object(reader, depth + 1)?),
        SNAPSHOT_ARRAY => TAtomic::Array(decode_complete_array(reader, depth + 1)?),
        SNAPSHOT_ITERABLE => {
            let key_type = Arc::new(decode_complete_union(reader, depth + 1)?);
            let value_type = Arc::new(decode_complete_union(reader, depth + 1)?);
            let intersection_types = decode_optional_complete_atomics(reader, depth + 1)?;
            TAtomic::Iterable(TIterable { key_type, value_type, intersection_types })
        }
        SNAPSHOT_RESOURCE => TAtomic::Resource(TResource::new(match reader.read_u8("resource state")? {
            0 => None,
            1 => Some(false),
            2 => Some(true),
            unknown => return Err(protocol(format!("unknown resource state {unknown}"))),
        })),
        SNAPSHOT_REFERENCE => TAtomic::Reference(decode_complete_reference(reader, depth + 1)?),
        SNAPSHOT_GENERIC_PARAMETER => TAtomic::GenericParameter(TGenericParameter {
            parameter_name: word(reader.read_bytes("generic parameter name")?),
            constraint: Arc::new(decode_complete_union(reader, depth + 1)?),
            defining_entity: decode_generic_parent(reader)?,
            intersection_types: decode_optional_complete_atomics(reader, depth + 1)?,
        }),
        SNAPSHOT_VARIABLE => TAtomic::Variable(word(reader.read_bytes("variable type name")?)),
        SNAPSHOT_CONDITIONAL => TAtomic::Conditional(TConditional::new(
            Arc::new(decode_complete_union(reader, depth + 1)?),
            Arc::new(decode_complete_union(reader, depth + 1)?),
            Arc::new(decode_complete_union(reader, depth + 1)?),
            Arc::new(decode_complete_union(reader, depth + 1)?),
            reader.read_bool("conditional negated flag")?,
        )),
        SNAPSHOT_DERIVED => TAtomic::Derived(decode_complete_derived(reader, depth + 1)?),
        SNAPSHOT_ALIAS => {
            TAtomic::Alias(TAlias::new(word(reader.read_bytes("alias class")?), word(reader.read_bytes("alias name")?)))
        }
        SNAPSHOT_NEVER => TAtomic::Never,
        SNAPSHOT_NULL => TAtomic::Null,
        SNAPSHOT_VOID => TAtomic::Void,
        SNAPSHOT_PLACEHOLDER => TAtomic::Placeholder,
        unknown => return Err(protocol(format!("unknown complete atomic type tag {unknown}"))),
    })
}

fn decode_complete_scalar(reader: &mut PayloadReader<'_>, depth: usize) -> Result<TScalar, ExternalAnalyzerError> {
    Ok(match reader.read_u8("complete scalar type kind")? {
        1 => TScalar::Generic,
        2 => TScalar::Numeric,
        3 => TScalar::ArrayKey,
        4 => TScalar::Bool(TBool::new(match reader.read_u8("boolean refinement")? {
            0 => None,
            1 => Some(false),
            2 => Some(true),
            unknown => return Err(protocol(format!("unknown boolean refinement {unknown}"))),
        })),
        5 => TScalar::Integer(match reader.read_u8("integer refinement")? {
            1 => TInteger::Literal(reader.read_u64("literal integer")? as i64),
            2 => TInteger::From(reader.read_u64("minimum integer")? as i64),
            3 => TInteger::To(reader.read_u64("maximum integer")? as i64),
            4 => {
                TInteger::Range(reader.read_u64("minimum integer")? as i64, reader.read_u64("maximum integer")? as i64)
            }
            5 => TInteger::Unspecified,
            6 => TInteger::UnspecifiedLiteral,
            unknown => return Err(protocol(format!("unknown integer refinement {unknown}"))),
        }),
        6 => TScalar::Float(match reader.read_u8("float refinement")? {
            1 => TFloat::Float,
            2 => TFloat::UnspecifiedLiteral,
            3 => TFloat::literal(f64::from_bits(reader.read_u64("literal float")?)),
            unknown => return Err(protocol(format!("unknown float refinement {unknown}"))),
        }),
        7 => {
            let literal = match reader.read_u8("string literal kind")? {
                0 => None,
                1 => Some(TStringLiteral::Unspecified),
                2 => Some(TStringLiteral::Value(word(reader.read_bytes("literal string")?))),
                unknown => return Err(protocol(format!("unknown string literal kind {unknown}"))),
            };
            let flags = reader.read_u8("string flags")?;
            let casing = match reader.read_u8("string casing")? {
                0 => TStringCasing::Unspecified,
                1 => TStringCasing::Lowercase,
                2 => TStringCasing::Uppercase,
                unknown => return Err(protocol(format!("unknown string casing {unknown}"))),
            };
            TScalar::String(TString::new(
                literal,
                flags & (1 << 0) != 0,
                flags & (1 << 1) != 0,
                flags & (1 << 2) != 0,
                flags & (1 << 3) != 0,
                casing,
            ))
        }
        8 => TScalar::ClassLikeString(decode_complete_class_like_string(reader, depth + 1)?),
        unknown => return Err(protocol(format!("unknown complete scalar type kind {unknown}"))),
    })
}

fn decode_complete_class_like_string(
    reader: &mut PayloadReader<'_>,
    depth: usize,
) -> Result<TClassLikeString, ExternalAnalyzerError> {
    Ok(match reader.read_u8("class-like string variant")? {
        1 => TClassLikeString::Any { kind: decode_class_like_string_kind(reader)? },
        2 => TClassLikeString::Generic {
            kind: decode_class_like_string_kind(reader)?,
            parameter_name: word(reader.read_bytes("class-like string parameter")?),
            defining_entity: decode_generic_parent(reader)?,
            constraint: Arc::new(decode_complete_atomic(reader, depth + 1)?),
        },
        3 => TClassLikeString::Literal { value: word(reader.read_bytes("literal class-like string")?) },
        4 => TClassLikeString::OfType {
            kind: decode_class_like_string_kind(reader)?,
            constraint: Arc::new(decode_complete_atomic(reader, depth + 1)?),
        },
        unknown => return Err(protocol(format!("unknown class-like string variant {unknown}"))),
    })
}

fn decode_class_like_string_kind(
    reader: &mut PayloadReader<'_>,
) -> Result<TClassLikeStringKind, ExternalAnalyzerError> {
    Ok(match reader.read_u8("class-like string kind")? {
        1 => TClassLikeStringKind::Class,
        2 => TClassLikeStringKind::Interface,
        3 => TClassLikeStringKind::Enum,
        4 => TClassLikeStringKind::Trait,
        unknown => return Err(protocol(format!("unknown class-like string kind {unknown}"))),
    })
}

fn decode_complete_callable(reader: &mut PayloadReader<'_>, depth: usize) -> Result<TCallable, ExternalAnalyzerError> {
    let kind = reader.read_u8("complete callable kind")?;
    if kind == 2 {
        return Ok(TCallable::Alias(decode_function_like_identifier(reader)?));
    }

    if kind != 1 {
        return Err(protocol(format!("unknown complete callable kind {kind}")));
    }

    let pure = reader.read_bool("callable pure flag")?;
    let closure = reader.read_bool("callable closure flag")?;
    let parameter_count = reader.read_count("callable parameters", MAXIMUM_TYPE_MEMBERS)?;
    let mut parameters = Vec::with_capacity(parameter_count);
    for _ in 0..parameter_count {
        let name = if reader.read_bool("callable parameter name presence")? {
            Some(VariableIdentifier(word(reader.read_bytes("callable parameter name")?)))
        } else {
            None
        };

        let parameter_type = if reader.read_bool("callable parameter type presence")? {
            Some(Arc::new(decode_complete_union(reader, depth + 1)?))
        } else {
            None
        };

        parameters.push(
            TCallableParameter::new(
                parameter_type,
                reader.read_bool("callable parameter by-reference flag")?,
                reader.read_bool("callable parameter variadic flag")?,
                reader.read_bool("callable parameter default flag")?,
            )
            .with_name(name),
        );
    }

    let return_type = if reader.read_bool("callable return type presence")? {
        Some(Arc::new(decode_complete_union(reader, depth + 1)?))
    } else {
        None
    };

    let source = if reader.read_bool("callable source presence")? {
        Some(decode_function_like_identifier(reader)?)
    } else {
        None
    };

    let constraint_count = reader.read_count("callable constraints", MAXIMUM_TYPE_MEMBERS)?;
    let mut constraints = Vec::with_capacity(constraint_count);
    for _ in 0..constraint_count {
        let name_count = reader.read_count("callable constraint names", MAXIMUM_TYPE_MEMBERS)?;
        let mut names = Vec::with_capacity(name_count);
        for _ in 0..name_count {
            names.push(VariableIdentifier(word(reader.read_bytes("callable constraint name")?)));
        }

        constraints.push(TCallableConstraint::new(
            names,
            Arc::new(decode_complete_union(reader, depth + 1)?),
            Arc::new(decode_complete_union(reader, depth + 1)?),
        ));
    }

    Ok(TCallable::Signature(
        TCallableSignature::new(pure, closure)
            .with_parameters(parameters)
            .with_return_type(return_type)
            .with_source(source)
            .with_constraints(constraints),
    ))
}

fn decode_complete_object(reader: &mut PayloadReader<'_>, depth: usize) -> Result<TObject, ExternalAnalyzerError> {
    Ok(match reader.read_u8("complete object kind")? {
        1 => TObject::Any,
        2 => {
            let name = word(reader.read_bytes("named object name")?);
            let type_parameters = decode_optional_complete_unions(reader, depth + 1)?;
            let variances = decode_optional_variances(reader)?;
            let is_static = reader.read_bool("named object static flag")?;
            let is_this = reader.read_bool("named object this flag")?;
            let intersection_types = decode_optional_complete_atomics(reader, depth + 1)?;
            let remapped_parameters = reader.read_bool("named object remapped parameters flag")?;
            TObject::Named(TNamedObject {
                name,
                type_parameters,
                variances,
                is_static,
                is_this,
                intersection_types,
                remapped_parameters,
            })
        }
        3 => {
            let name = word(reader.read_bytes("enum name")?);
            match reader.read_optional_string("enum case")? {
                Some(case) => TObject::new_enum_case(name, word(case)),
                None => TObject::new_enum(name),
            }
        }
        4 => {
            let sealed = reader.read_bool("object shape sealed flag")?;
            let property_count = reader.read_count("object shape properties", MAXIMUM_TYPE_MEMBERS)?;
            let mut known_properties = BTreeMap::new();
            for _ in 0..property_count {
                let name = word(reader.read_bytes("object shape property name")?);
                let optional = reader.read_bool("object shape property optional flag")?;
                known_properties.insert(name, (optional, decode_complete_union(reader, depth + 1)?));
            }
            TObject::WithProperties(TObjectWithProperties { known_properties, sealed })
        }
        5 => TObject::HasMethod(TObjectHasMethod {
            method: word(reader.read_bytes("required method name")?),
            intersection_types: decode_optional_complete_atomics(reader, depth + 1)?,
        }),
        6 => TObject::HasProperty(TObjectHasProperty {
            property: word(reader.read_bytes("required property name")?),
            intersection_types: decode_optional_complete_atomics(reader, depth + 1)?,
        }),
        unknown => return Err(protocol(format!("unknown complete object kind {unknown}"))),
    })
}

fn decode_complete_array(reader: &mut PayloadReader<'_>, depth: usize) -> Result<TArray, ExternalAnalyzerError> {
    Ok(match reader.read_u8("complete array kind")? {
        1 => {
            let element_type = Arc::new(decode_complete_union(reader, depth + 1)?);
            let known_elements = if reader.read_bool("known list elements presence")? {
                let count = reader.read_count("known list elements", MAXIMUM_TYPE_MEMBERS)?;
                let mut elements = BTreeMap::new();
                for _ in 0..count {
                    let index = usize::try_from(reader.read_u64("known list element index")?)
                        .map_err(|_| protocol("known list element index exceeds usize::MAX"))?;
                    let optional = reader.read_bool("known list element optional flag")?;
                    elements.insert(index, (optional, decode_complete_union(reader, depth + 1)?));
                }

                Some(elements)
            } else {
                None
            };
            let known_count = if reader.read_bool("known list count presence")? {
                Some(
                    usize::try_from(reader.read_u64("known list count")?)
                        .map_err(|_| protocol("known list count exceeds usize::MAX"))?,
                )
            } else {
                None
            };

            let non_empty = reader.read_bool("list non-empty flag")?;
            TArray::List(TList { element_type, known_elements, known_count, non_empty })
        }
        2 => {
            let known_items = if reader.read_bool("known array items presence")? {
                let count = reader.read_count("known array items", MAXIMUM_TYPE_MEMBERS)?;
                let mut items = BTreeMap::new();
                for _ in 0..count {
                    let key = decode_array_key(reader)?;
                    let optional = reader.read_bool("known array item optional flag")?;
                    items.insert(key, (optional, decode_complete_union(reader, depth + 1)?));
                }
                Some(items)
            } else {
                None
            };

            let parameters = if reader.read_bool("array parameters presence")? {
                Some((
                    Arc::new(decode_complete_union(reader, depth + 1)?),
                    Arc::new(decode_complete_union(reader, depth + 1)?),
                ))
            } else {
                None
            };

            let non_empty = reader.read_bool("array non-empty flag")?;
            TArray::Keyed(TKeyedArray { known_items, parameters, non_empty })
        }
        unknown => return Err(protocol(format!("unknown complete array kind {unknown}"))),
    })
}

fn decode_array_key(reader: &mut PayloadReader<'_>) -> Result<ArrayKey, ExternalAnalyzerError> {
    Ok(match reader.read_u8("array key kind")? {
        1 => ArrayKey::Integer(reader.read_u64("integer array key")? as i64),
        2 => ArrayKey::String(word(reader.read_bytes("string array key")?)),
        3 => ArrayKey::ClassLikeConstant {
            class_like_name: word(reader.read_bytes("array key class")?),
            constant_name: word(reader.read_bytes("array key constant")?),
        },
        unknown => return Err(protocol(format!("unknown array key kind {unknown}"))),
    })
}

fn decode_complete_reference(
    reader: &mut PayloadReader<'_>,
    depth: usize,
) -> Result<TReference, ExternalAnalyzerError> {
    Ok(match reader.read_u8("complete reference kind")? {
        1 => TReference::Symbol {
            name: word(reader.read_bytes("referenced symbol name")?),
            parameters: decode_optional_complete_unions(reader, depth + 1)?,
            variances: decode_optional_variances(reader)?,
            intersection_types: decode_optional_complete_atomics(reader, depth + 1)?,
        },
        2 => TReference::Member {
            class_like_name: word(reader.read_bytes("reference member class")?),
            member_selector: match reader.read_u8("reference member selector")? {
                1 => TReferenceMemberSelector::Wildcard,
                2 => TReferenceMemberSelector::Identifier(word(reader.read_bytes("reference member")?)),
                3 => TReferenceMemberSelector::StartsWith(word(reader.read_bytes("reference member prefix")?)),
                4 => TReferenceMemberSelector::EndsWith(word(reader.read_bytes("reference member suffix")?)),
                unknown => return Err(protocol(format!("unknown member reference selector {unknown}"))),
            },
        },
        3 => TReference::Global {
            selector: match reader.read_u8("global reference selector")? {
                3 => TGlobalReferenceSelector::StartsWith(word(reader.read_bytes("global reference prefix")?)),
                4 => TGlobalReferenceSelector::EndsWith(word(reader.read_bytes("global reference suffix")?)),
                unknown => return Err(protocol(format!("unknown global reference selector {unknown}"))),
            },
        },
        unknown => return Err(protocol(format!("unknown complete reference kind {unknown}"))),
    })
}

fn decode_complete_derived(reader: &mut PayloadReader<'_>, depth: usize) -> Result<TDerived, ExternalAnalyzerError> {
    Ok(match reader.read_u8("complete derived type kind")? {
        1 => TDerived::KeyOf(TKeyOf::new(Arc::new(decode_complete_union(reader, depth + 1)?))),
        2 => TDerived::ValueOf(TValueOf::new(Arc::new(decode_complete_union(reader, depth + 1)?))),
        3 => {
            let count = reader.read_count("int-mask values", MAXIMUM_TYPE_MEMBERS)?;
            let mut values = Vec::with_capacity(count);
            for _ in 0..count {
                values.push(decode_complete_union(reader, depth + 1)?);
            }

            TDerived::IntMask(TIntMask::new(values))
        }
        4 => TDerived::IntMaskOf(TIntMaskOf::new(Arc::new(decode_complete_union(reader, depth + 1)?))),
        5 => {
            let visibility = match reader.read_u8("properties-of visibility")? {
                0 => None,
                1 => Some(Visibility::Public),
                2 => Some(Visibility::Protected),
                3 => Some(Visibility::Private),
                unknown => return Err(protocol(format!("unknown properties-of visibility {unknown}"))),
            };
            TDerived::PropertiesOf(TPropertiesOf {
                visibility,
                target_type: Arc::new(decode_complete_union(reader, depth + 1)?),
            })
        }
        6 => TDerived::IndexAccess(TIndexAccess::new(
            decode_complete_union(reader, depth + 1)?,
            decode_complete_union(reader, depth + 1)?,
        )),
        7 => TDerived::New(TNew::new(Arc::new(decode_complete_union(reader, depth + 1)?))),
        8 => TDerived::TemplateType(TTemplateType::new(
            Arc::new(decode_complete_union(reader, depth + 1)?),
            Arc::new(decode_complete_union(reader, depth + 1)?),
            Arc::new(decode_complete_union(reader, depth + 1)?),
        )),
        9 => {
            let mut intersection = TDerivedIntersection::new(decode_complete_union(reader, depth + 1)?);
            let count = reader.read_count("derived intersection types", MAXIMUM_TYPE_MEMBERS)?;
            for _ in 0..count {
                intersection.add_intersection_type(decode_complete_atomic(reader, depth + 1)?);
            }
            TDerived::Intersection(intersection)
        }
        unknown => return Err(protocol(format!("unknown complete derived type kind {unknown}"))),
    })
}

fn decode_optional_complete_unions(
    reader: &mut PayloadReader<'_>,
    depth: usize,
) -> Result<Option<Vec<TUnion>>, ExternalAnalyzerError> {
    if !reader.read_bool("nested union types presence")? {
        return Ok(None);
    }

    let count = reader.read_count("nested union types", MAXIMUM_TYPE_MEMBERS)?;
    let mut unions = Vec::with_capacity(count);
    for _ in 0..count {
        unions.push(decode_complete_union(reader, depth + 1)?);
    }

    Ok(Some(unions))
}

fn decode_optional_complete_atomics(
    reader: &mut PayloadReader<'_>,
    depth: usize,
) -> Result<Option<Vec<TAtomic>>, ExternalAnalyzerError> {
    if !reader.read_bool("intersection types presence")? {
        return Ok(None);
    }

    let count = reader.read_count("intersection types", MAXIMUM_TYPE_MEMBERS)?;
    let mut atomics = Vec::with_capacity(count);
    for _ in 0..count {
        atomics.push(decode_complete_atomic(reader, depth + 1)?);
    }

    Ok(Some(atomics))
}

fn decode_optional_variances(reader: &mut PayloadReader<'_>) -> Result<Option<Vec<Variance>>, ExternalAnalyzerError> {
    if !reader.read_bool("variances presence")? {
        return Ok(None);
    }

    let count = reader.read_count("variances", MAXIMUM_TYPE_MEMBERS)?;
    let mut variances = Vec::with_capacity(count);
    for _ in 0..count {
        variances.push(match reader.read_u8("variance")? {
            1 => Variance::Invariant,
            2 => Variance::Covariant,
            3 => Variance::Contravariant,
            4 => Variance::Bivariant,
            unknown => return Err(protocol(format!("unknown variance {unknown}"))),
        });
    }

    Ok(Some(variances))
}

fn decode_generic_parent(reader: &mut PayloadReader<'_>) -> Result<GenericParent, ExternalAnalyzerError> {
    Ok(match reader.read_u8("generic parent kind")? {
        1 => GenericParent::ClassLike(word(reader.read_bytes("generic parent class")?)),
        2 => GenericParent::FunctionLike((
            word(reader.read_bytes("generic parent function-like")?),
            word(reader.read_bytes("generic parent member")?),
        )),
        unknown => return Err(protocol(format!("unknown generic parent kind {unknown}"))),
    })
}

fn decode_function_like_identifier(
    reader: &mut PayloadReader<'_>,
) -> Result<FunctionLikeIdentifier, ExternalAnalyzerError> {
    Ok(match reader.read_u8("function-like identifier kind")? {
        1 => FunctionLikeIdentifier::Function(word(reader.read_bytes("function name")?)),
        2 => FunctionLikeIdentifier::Method(
            word(reader.read_bytes("method class")?),
            word(reader.read_bytes("method name")?),
        ),
        3 => FunctionLikeIdentifier::Closure(word(reader.read_bytes("closure name")?)),
        unknown => return Err(protocol(format!("unknown function-like identifier kind {unknown}"))),
    })
}

pub(super) fn message_writer(kind: u16) -> PayloadWriter {
    let mut writer = PayloadWriter::with_capacity(INITIAL_MESSAGE_CAPACITY);
    writer.write_raw(&ANALYZER_PROTOCOL_MAGIC);
    writer.write_u16(ANALYZER_PROTOCOL_MAJOR);
    writer.write_u16(ANALYZER_PROTOCOL_MINOR);
    writer.write_u16(kind);
    writer.write_u16(0);
    writer
}

pub(super) fn message_reader(payload: &[u8], expected_kind: u16) -> Result<PayloadReader<'_>, ExternalAnalyzerError> {
    if payload.len() < HEADER_LENGTH {
        return Err(protocol("analyzer payload is shorter than its header"));
    }

    let mut reader = PayloadReader::new(payload);
    if reader.read_array::<4>("analyzer magic")? != ANALYZER_PROTOCOL_MAGIC {
        return Err(protocol("invalid analyzer payload magic"));
    }

    let major = reader.read_u16("analyzer protocol major")?;
    let _minor = reader.read_u16("analyzer protocol minor")?;
    if major != ANALYZER_PROTOCOL_MAJOR {
        return Err(protocol(format!("unsupported analyzer protocol major {major}")));
    }

    let kind = reader.read_u16("analyzer message kind")?;
    if kind != expected_kind {
        return Err(protocol(format!("expected analyzer message kind {expected_kind}, received {kind}")));
    }

    if reader.read_u16("analyzer reserved bits")? != 0 {
        return Err(protocol("analyzer message reserved bits are non-zero"));
    }

    Ok(reader)
}

pub(super) fn message_kind(payload: &[u8]) -> Result<u16, ExternalAnalyzerError> {
    if payload.len() < HEADER_LENGTH {
        return Err(protocol("analyzer message is shorter than its header"));
    }

    let mut reader = PayloadReader::new(payload);
    let magic = reader.read_array::<4>("analyzer protocol magic")?;
    if magic != ANALYZER_PROTOCOL_MAGIC {
        return Err(protocol("invalid analyzer protocol magic"));
    }

    let major = reader.read_u16("analyzer protocol major version")?;
    let minor = reader.read_u16("analyzer protocol minor version")?;
    if major != ANALYZER_PROTOCOL_MAJOR || minor != ANALYZER_PROTOCOL_MINOR {
        return Err(protocol(format!("unsupported analyzer protocol version {major}.{minor}")));
    }

    let kind = reader.read_u16("analyzer message kind")?;
    let reserved = reader.read_u16("analyzer reserved header field")?;
    if reserved != 0 {
        return Err(protocol("analyzer reserved header field is non-zero"));
    }

    Ok(kind)
}

fn non_empty<T>(value: T, field: &str) -> Result<T, ExternalAnalyzerError>
where
    T: AsRef<[u8]>,
{
    if value.as_ref().is_empty() { Err(protocol(format!("{field} cannot be empty"))) } else { Ok(value) }
}

fn validate_method_pattern(pattern: &[u8]) -> Result<(), ExternalAnalyzerError> {
    if let Some(star) = pattern.iter().position(|byte| *byte == b'*')
        && star + 1 != pattern.len()
    {
        return Err(protocol("method target wildcards are only allowed as the final byte"));
    }

    Ok(())
}

fn validate_provider_indices(
    indices: impl IntoIterator<Item = u16>,
    provider_kind: &str,
) -> Result<(), ExternalAnalyzerError> {
    for (expected, actual) in indices.into_iter().enumerate() {
        if actual as usize != expected {
            return Err(protocol(format!(
                "{provider_kind} indices must be dense and ordered; expected {expected}, received {actual}"
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
pub(super) mod testing {
    use super::*;

    #[test]
    fn complete_snapshots_accept_deep_inferred_types() {
        let mut ty = get_int();
        for _ in 0..72 {
            ty = get_list(ty);
        }

        let mut writer = PayloadWriter::new();
        let mut types = Vec::new();
        encode_union_snapshot(&mut writer, &ty, &mut types, 0).unwrap();

        assert_eq!(types.len(), 73);
        assert!(!writer.finish().is_empty());
    }

    #[test]
    fn initialization_rejects_duplicate_stub_filenames() {
        let mut writer = message_writer(INITIALIZE_RESPONSE);
        writer.write_u32(1);
        writer.write_u16(0);
        writer.write_u32(2);
        for _ in 0..2 {
            writer.write_bytes(b"duplicate.php").unwrap();
            writer.write_bytes(b"<?php").unwrap();
        }

        let result = decode_initialization_response(&writer.finish(), &[0]);
        assert!(matches!(result, Err(ExternalAnalyzerError::Protocol(message)) if message.contains("more than once")));
    }

    pub fn registration_response() -> Vec<u8> {
        registration_response_with_lifecycle(0)
    }

    pub fn registration_response_with_initialization() -> Vec<u8> {
        registration_response_with_lifecycle(8)
    }

    fn registration_response_with_lifecycle(lifecycle: u8) -> Vec<u8> {
        let mut writer = message_writer(DESCRIBE_RESPONSE);
        writer.write_u32(1);
        writer.write_string("demo/extension").unwrap();
        writer.write_string("Demo").unwrap();
        writer.write_string("1.0.0").unwrap();
        writer.write_u32(1);
        writer.write_string("demo").unwrap();
        writer.write_string("Demo analyzer").unwrap();
        writer.write_string("Test provider").unwrap();
        writer.write_bool(true);
        writer.write_u8(lifecycle);
        writer.write_u32(1);
        writer.write_string("example").unwrap();
        writer.write_u32(1);
        writer.write_u16(0);
        writer.write_u32(1);
        writer.write_u8(TARGET_EXACT);
        writer.write_bytes(b"demo_service").unwrap();
        writer.write_u32(0);
        writer.finish()
    }

    pub fn initialization_response(filename: &[u8], contents: &[u8]) -> Vec<u8> {
        let mut writer = message_writer(INITIALIZE_RESPONSE);
        writer.write_u32(1);
        writer.write_u16(0);
        writer.write_u32(1);
        writer.write_bytes(filename).unwrap();
        writer.write_bytes(contents).unwrap();
        writer.finish()
    }

    pub fn named_object_response(name: &str) -> Vec<u8> {
        let mut writer = message_writer(RETURN_TYPE_RESPONSE);
        writer.write_bool(true);
        writer.write_u8(TYPE_NAMED_OBJECT);
        writer.write_bytes(name.as_bytes()).unwrap();
        writer.write_u32(0);
        writer.finish()
    }

    pub fn reference_response(handle: u32) -> Vec<u8> {
        let mut writer = message_writer(RETURN_TYPE_RESPONSE);
        writer.write_bool(true);
        writer.write_u8(TYPE_REFERENCE);
        writer.write_u32(handle);
        writer.finish()
    }

    fn refined_scalar_response(tag: u8) -> Vec<u8> {
        let mut writer = message_writer(RETURN_TYPE_RESPONSE);
        writer.write_bool(true);
        writer.write_u8(tag);
        writer.finish()
    }

    pub fn non_negative_int_response() -> Vec<u8> {
        refined_scalar_response(TYPE_NON_NEGATIVE_INT)
    }

    pub fn non_empty_string_response() -> Vec<u8> {
        refined_scalar_response(TYPE_NON_EMPTY_STRING)
    }

    pub fn complete_non_empty_string_list_response() -> Vec<u8> {
        let mut writer = message_writer(RETURN_TYPE_RESPONSE);
        writer.write_bool(true);
        writer.write_u8(TYPE_COMPLETE);
        writer.write_u16(0);
        writer.write_u32(1);
        writer.write_u8(SNAPSHOT_ARRAY);
        writer.write_u8(1);
        writer.write_u16(0);
        writer.write_u32(1);
        writer.write_u8(SNAPSHOT_SCALAR);
        writer.write_u8(7);
        writer.write_u8(0);
        writer.write_u8(0);
        writer.write_u8(0);
        writer.write_bool(false);
        writer.write_bool(false);
        writer.write_bool(true);
        writer.finish()
    }
}
