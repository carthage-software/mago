use std::collections::BTreeMap;

use mago_codex::assertion::Assertion;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::metadata::attribute::AttributeMetadata;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::metadata::class_like::TemplateTypes;
use mago_codex::metadata::class_like_constant::ClassLikeConstantMetadata;
use mago_codex::metadata::constant::ConstantMetadata;
use mago_codex::metadata::enum_case::EnumCaseMetadata;
use mago_codex::metadata::function_like::FunctionLikeKind;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_codex::metadata::parameter::FunctionLikeParameterMetadata;
use mago_codex::metadata::property::PropertyMetadata;
use mago_codex::metadata::property_hook::PropertyHookMetadata;
use mago_codex::metadata::ttype::TypeMetadata;
use mago_codex::metadata::version_constraint::VersionConstraint;
use mago_codex::symbol::SymbolKind;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::key::ArrayKey;
use mago_codex::ttype::template::variance::Variance;
use mago_codex::ttype::union::TUnion;
use mago_codex::visibility::Visibility;
use mago_extension::PayloadReader;
use mago_extension::PayloadWriter;
use mago_span::Span;
use mago_word::Word;

use crate::external::ExternalAnalysisSession;
use crate::external::error::ExternalAnalyzerError;
use crate::external::error::protocol;
use crate::external::protocol::decode_function_like_identifier;
use crate::external::protocol::encode_generic_parent;
use crate::external::protocol::encode_union_snapshot;

pub(super) const CODEBASE_QUERY_REQUEST: u16 = 4;
pub(super) const CODEBASE_QUERY_RESPONSE: u16 = 0x8004;

const GET_CLASS_LIKES: u8 = 1;
const GET_FUNCTIONS: u8 = 2;
const GET_METHODS: u8 = 3;
const GET_CONSTANTS: u8 = 4;
const GET_PROPERTIES: u8 = 5;
const GET_CLASS_CONSTANTS: u8 = 6;
const GET_ENUM_CASES: u8 = 7;
const LIST_CLASS_LIKES: u8 = 8;
const LIST_FUNCTIONS: u8 = 9;
const LIST_CONSTANTS: u8 = 10;
const GET_DECLARING_METHODS: u8 = 11;
const GET_DECLARING_PROPERTIES: u8 = 12;
const CHECK_EXISTENCE: u8 = 13;
const CHECK_MEMBER_EXISTENCE: u8 = 14;
const GET_CLASS_LIKE_RELATIONS: u8 = 15;
const GET_MAGIC_PROPERTIES: u8 = 16;
const GET_DECLARING_MAGIC_PROPERTIES: u8 = 17;
const GET_FUNCTION_LIKES: u8 = 18;

const ANY_CLASS_LIKE: u8 = 0;
const CLASS: u8 = 1;
const INTERFACE: u8 = 2;
const TRAIT: u8 = 3;
const ENUM: u8 = 4;

const MAXIMUM_QUERIES: usize = 0x0001_0000;

const EXISTS_CLASS: u8 = 1;
const EXISTS_INTERFACE: u8 = 2;
const EXISTS_TRAIT: u8 = 3;
const EXISTS_ENUM: u8 = 4;
const EXISTS_CLASS_LIKE: u8 = 5;
const EXISTS_NAMESPACE: u8 = 6;
const EXISTS_FUNCTION: u8 = 7;
const EXISTS_CONSTANT: u8 = 8;
const EXISTS_CLASS_OR_TRAIT: u8 = 9;
const EXISTS_CLASS_OR_INTERFACE: u8 = 10;

const EXISTS_METHOD: u8 = 1;
const EXISTS_PROPERTY: u8 = 2;
const EXISTS_CLASS_CONSTANT: u8 = 3;
const EXISTS_ENUM_CASE: u8 = 4;
const EXISTS_MAGIC_PROPERTY: u8 = 5;

const DIRECT_DESCENDANTS: u8 = 1;
const ALL_DESCENDANTS: u8 = 2;
const ALL_ANCESTORS: u8 = 3;

pub(super) fn handle_query(
    reader: &mut PayloadReader<'_>,
    codebase: &CodebaseMetadata,
    session: &ExternalAnalysisSession,
    writer: &mut PayloadWriter,
) -> Result<(), ExternalAnalyzerError> {
    let generation = reader.read_u64("codebase generation")?;
    if generation != session.generation() {
        return Err(protocol(format!(
            "metadata query targets generation {generation}, but the active generation is {}",
            session.generation()
        )));
    }

    writer.write_u64(generation);
    let operation = reader.read_u8("codebase query operation")?;
    writer.write_u8(operation);
    match operation {
        GET_CLASS_LIKES => query_class_likes(reader, writer, codebase, session),
        GET_FUNCTIONS => query_names(reader, writer, |name, writer| {
            write_optional(writer, codebase.get_function(name), |writer, metadata| {
                write_function_like(writer, metadata, session)
            })
        }),
        GET_METHODS => query_members(reader, writer, |class, member, writer| {
            write_optional(writer, codebase.get_method(class, member), |writer, metadata| {
                write_function_like(writer, metadata, session)
            })
        }),
        GET_CONSTANTS => query_names(reader, writer, |name, writer| {
            write_optional(writer, codebase.get_constant(name), |writer, metadata| {
                write_constant(writer, metadata, session)
            })
        }),
        GET_PROPERTIES => query_members(reader, writer, |class, member, writer| {
            write_optional(writer, codebase.get_property(class, member), |writer, metadata| {
                write_property(writer, metadata, session)
            })
        }),
        GET_CLASS_CONSTANTS => query_members(reader, writer, |class, member, writer| {
            write_optional(writer, codebase.get_class_constant(class, member), |writer, metadata| {
                write_class_constant(writer, metadata, session)
            })
        }),
        GET_ENUM_CASES => query_members(reader, writer, |class, member, writer| {
            write_optional(writer, codebase.get_enum_case(class, member), |writer, metadata| {
                write_enum_case(writer, metadata, session)
            })
        }),
        LIST_CLASS_LIKES => list_class_likes(reader, writer, codebase),
        LIST_FUNCTIONS => list_functions(reader, writer, codebase),
        LIST_CONSTANTS => list_constants(reader, writer, codebase),
        GET_DECLARING_METHODS => query_members(reader, writer, |class, member, writer| {
            write_optional(writer, codebase.get_declaring_method(class, member), |writer, metadata| {
                write_function_like(writer, metadata, session)
            })
        }),
        GET_DECLARING_PROPERTIES => query_members(reader, writer, |class, member, writer| {
            write_optional(writer, codebase.get_declaring_property(class, member), |writer, metadata| {
                write_property(writer, metadata, session)
            })
        }),
        CHECK_EXISTENCE => check_existence(reader, writer, codebase),
        CHECK_MEMBER_EXISTENCE => check_member_existence(reader, writer, codebase),
        GET_CLASS_LIKE_RELATIONS => get_class_like_relations(reader, writer, codebase),
        GET_MAGIC_PROPERTIES => query_members(reader, writer, |class, member, writer| {
            write_optional(writer, codebase.get_magic_property(class, member), |writer, metadata| {
                write_property(writer, metadata, session)
            })
        }),
        GET_DECLARING_MAGIC_PROPERTIES => query_members(reader, writer, |class, member, writer| {
            write_optional(writer, codebase.get_declaring_magic_property(class, member), |writer, metadata| {
                write_property(writer, metadata, session)
            })
        }),
        GET_FUNCTION_LIKES => query_function_likes(reader, writer, codebase, session),
        unknown => Err(protocol(format!("unknown codebase query operation {unknown}"))),
    }
}

fn query_function_likes(
    reader: &mut PayloadReader<'_>,
    writer: &mut PayloadWriter,
    codebase: &CodebaseMetadata,
    session: &ExternalAnalysisSession,
) -> Result<(), ExternalAnalyzerError> {
    let count = reader.read_count("function-like metadata queries", MAXIMUM_QUERIES)?;
    writer.write_u32(count as u32);
    for _ in 0..count {
        let identifier = decode_function_like_identifier(reader)?;
        write_optional(writer, codebase.get_function_like(&identifier), |writer, metadata| {
            write_function_like(writer, metadata, session)
        })?;
    }
    Ok(())
}

fn get_class_like_relations(
    reader: &mut PayloadReader<'_>,
    writer: &mut PayloadWriter,
    codebase: &CodebaseMetadata,
) -> Result<(), ExternalAnalyzerError> {
    let relation = reader.read_u8("class-like relation")?;
    if !(DIRECT_DESCENDANTS..=ALL_ANCESTORS).contains(&relation) {
        return Err(protocol(format!("unknown class-like relation {relation}")));
    }

    writer.write_u8(relation);
    query_names(reader, writer, |name, writer| {
        match relation {
            DIRECT_DESCENDANTS => {
                let descendants = codebase
                    .get_class_like(name)
                    .and_then(|metadata| codebase.direct_classlike_descendants.get(&metadata.name));
                write_words(writer, descendants.into_iter().flatten().copied())?;
            }
            ALL_DESCENDANTS => write_words(writer, codebase.get_class_descendants(name))?,
            ALL_ANCESTORS => write_words(writer, codebase.get_class_ancestors(name))?,
            _ => unreachable!(),
        }

        Ok(())
    })
}

fn check_member_existence(
    reader: &mut PayloadReader<'_>,
    writer: &mut PayloadWriter,
    codebase: &CodebaseMetadata,
) -> Result<(), ExternalAnalyzerError> {
    let predicate = reader.read_u8("member existence predicate")?;
    if !(EXISTS_METHOD..=EXISTS_MAGIC_PROPERTY).contains(&predicate) {
        return Err(protocol(format!("unknown member existence predicate {predicate}")));
    }

    writer.write_u8(predicate);
    query_members(reader, writer, |class, member, writer| {
        writer.write_bool(match predicate {
            EXISTS_METHOD => codebase.method_exists(class, member),
            EXISTS_PROPERTY => codebase.property_exists(class, member),
            EXISTS_CLASS_CONSTANT => codebase.class_constant_exists(class, member),
            EXISTS_ENUM_CASE => codebase.get_enum_case(class, member).is_some(),
            EXISTS_MAGIC_PROPERTY => codebase.magic_property_exists(class, member),
            _ => unreachable!(),
        });

        Ok(())
    })
}

fn check_existence(
    reader: &mut PayloadReader<'_>,
    writer: &mut PayloadWriter,
    codebase: &CodebaseMetadata,
) -> Result<(), ExternalAnalyzerError> {
    let predicate = reader.read_u8("existence predicate")?;
    if !(EXISTS_CLASS..=EXISTS_CLASS_OR_INTERFACE).contains(&predicate) {
        return Err(protocol(format!("unknown existence predicate {predicate}")));
    }

    writer.write_u8(predicate);
    query_names(reader, writer, |name, writer| {
        writer.write_bool(match predicate {
            EXISTS_CLASS => codebase.class_exists(name),
            EXISTS_INTERFACE => codebase.interface_exists(name),
            EXISTS_TRAIT => codebase.trait_exists(name),
            EXISTS_ENUM => codebase.enum_exists(name),
            EXISTS_CLASS_LIKE => codebase.class_like_exists(name),
            EXISTS_NAMESPACE => codebase.namespace_exists(name),
            EXISTS_FUNCTION => codebase.function_exists(name),
            EXISTS_CONSTANT => codebase.constant_exists(name),
            EXISTS_CLASS_OR_TRAIT => codebase.class_or_trait_exists(name),
            EXISTS_CLASS_OR_INTERFACE => codebase.class_or_interface_exists(name),
            _ => unreachable!(),
        });

        Ok(())
    })
}

fn list_class_likes(
    reader: &mut PayloadReader<'_>,
    writer: &mut PayloadWriter,
    codebase: &CodebaseMetadata,
) -> Result<(), ExternalAnalyzerError> {
    let filter = reader.read_u8("class-like kind filter")?;
    if filter > ENUM {
        return Err(protocol(format!("unknown class-like kind filter {filter}")));
    }

    writer.write_u8(filter);
    write_words(
        writer,
        codebase
            .class_likes
            .values()
            .filter(|metadata| class_like_matches(metadata.kind, filter))
            .map(|metadata| metadata.original_name),
    )
}

fn list_functions(
    _reader: &mut PayloadReader<'_>,
    writer: &mut PayloadWriter,
    codebase: &CodebaseMetadata,
) -> Result<(), ExternalAnalyzerError> {
    write_words(
        writer,
        codebase
            .function_likes
            .iter()
            .filter(|((scope, _), metadata)| scope.is_empty() && metadata.kind == FunctionLikeKind::Function)
            .map(|(_, metadata)| metadata.original_name),
    )
}

fn list_constants(
    _reader: &mut PayloadReader<'_>,
    writer: &mut PayloadWriter,
    codebase: &CodebaseMetadata,
) -> Result<(), ExternalAnalyzerError> {
    write_words(writer, codebase.constants.values().map(|metadata| metadata.name))
}

fn query_class_likes(
    reader: &mut PayloadReader<'_>,
    writer: &mut PayloadWriter,
    codebase: &CodebaseMetadata,
    session: &ExternalAnalysisSession,
) -> Result<(), ExternalAnalyzerError> {
    let filter = reader.read_u8("class-like kind filter")?;
    if filter > ENUM {
        return Err(protocol(format!("unknown class-like kind filter {filter}")));
    }

    writer.write_u8(filter);
    query_names(reader, writer, |name, writer| {
        let metadata = match filter {
            ANY_CLASS_LIKE => codebase.get_class_like(name),
            CLASS => codebase.get_class(name),
            INTERFACE => codebase.get_interface(name),
            TRAIT => codebase.get_trait(name),
            ENUM => codebase.get_enum(name),
            _ => unreachable!(),
        };

        write_optional(writer, metadata, |writer, metadata| write_class_like(writer, metadata, session))
    })
}

fn query_names<F>(
    reader: &mut PayloadReader<'_>,
    writer: &mut PayloadWriter,
    mut query: F,
) -> Result<(), ExternalAnalyzerError>
where
    F: FnMut(&[u8], &mut PayloadWriter) -> Result<(), ExternalAnalyzerError>,
{
    let count = reader.read_count("metadata query names", MAXIMUM_QUERIES)?;
    writer.write_u32(count as u32);
    for _ in 0..count {
        query(reader.read_bytes("metadata query name")?, writer)?;
    }

    Ok(())
}

fn query_members<F>(
    reader: &mut PayloadReader<'_>,
    writer: &mut PayloadWriter,
    mut query: F,
) -> Result<(), ExternalAnalyzerError>
where
    F: FnMut(&[u8], &[u8], &mut PayloadWriter) -> Result<(), ExternalAnalyzerError>,
{
    let count = reader.read_count("metadata query members", MAXIMUM_QUERIES)?;
    writer.write_u32(count as u32);
    for _ in 0..count {
        let class = reader.read_bytes("metadata query class")?;
        let member = reader.read_bytes("metadata query member")?;
        query(class, member, writer)?;
    }

    Ok(())
}

fn write_optional<T, F>(writer: &mut PayloadWriter, value: Option<&T>, encode: F) -> Result<(), ExternalAnalyzerError>
where
    F: FnOnce(&mut PayloadWriter, &T) -> Result<(), ExternalAnalyzerError>,
{
    writer.write_bool(value.is_some());
    if let Some(value) = value {
        encode(writer, value)?;
    }

    Ok(())
}

fn write_class_like(
    writer: &mut PayloadWriter,
    metadata: &ClassLikeMetadata,
    session: &ExternalAnalysisSession,
) -> Result<(), ExternalAnalyzerError> {
    writer.write_bytes(metadata.name.as_bytes())?;
    writer.write_bytes(metadata.original_name.as_bytes())?;
    writer.write_u8(symbol_kind(metadata.kind));
    write_location(writer, metadata.span, session)?;
    write_optional_location(writer, metadata.name_span, session)?;
    writer.write_u64(metadata.flags.bits());
    write_optional_word(writer, metadata.direct_parent_class)?;
    write_words(writer, metadata.direct_parent_interfaces.iter().copied())?;
    write_words(writer, metadata.all_parent_interfaces.iter().copied())?;
    write_words(writer, metadata.all_parent_classes.iter().copied())?;
    write_words(writer, metadata.require_extends.iter().copied())?;
    write_words(writer, metadata.require_implements.iter().copied())?;
    write_words(writer, metadata.used_traits.iter().copied())?;
    write_words(writer, metadata.methods.iter().copied())?;
    write_words(writer, metadata.pseudo_methods.iter().copied())?;
    write_words(writer, metadata.static_pseudo_methods.iter().copied())?;
    write_words(writer, metadata.properties.keys().copied())?;
    write_words(writer, metadata.magic_properties.keys().copied())?;
    write_words(writer, metadata.constants.keys().copied())?;
    write_words(writer, metadata.enum_cases.keys().copied())?;
    write_optional_words(writer, metadata.child_class_likes.as_ref().map(|words| words.iter().copied()))?;
    write_optional_words(writer, metadata.permitted_inheritors.as_ref().map(|words| words.iter().copied()))?;
    write_templates(
        writer,
        &metadata.template_types,
        Some(&metadata.template_variance),
        Some(&metadata.template_readonly),
    )?;
    write_attributes(writer, &metadata.attributes, session)?;
    write_type_aliases(writer, &metadata.type_aliases, session)?;
    write_mixins(writer, &metadata.mixins)?;
    write_optional_atomic(writer, metadata.enum_type.as_ref())?;
    write_optional_bool(writer, metadata.has_sealed_methods);
    write_optional_bool(writer, metadata.has_sealed_properties);
    write_version_constraint(writer, &metadata.version_constraint);
    Ok(())
}

fn write_function_like(
    writer: &mut PayloadWriter,
    metadata: &FunctionLikeMetadata,
    session: &ExternalAnalysisSession,
) -> Result<(), ExternalAnalyzerError> {
    writer.write_u8(match metadata.kind {
        FunctionLikeKind::Function => 1,
        FunctionLikeKind::Method => 2,
        FunctionLikeKind::Closure => 3,
        FunctionLikeKind::ArrowFunction => 4,
    });

    writer.write_bytes(metadata.name.as_bytes())?;
    writer.write_bytes(metadata.original_name.as_bytes())?;
    write_location(writer, metadata.span, session)?;
    write_optional_location(writer, metadata.name_span, session)?;
    writer.write_u32(metadata.parameters.len() as u32);
    for parameter in &metadata.parameters {
        write_parameter(writer, parameter, session)?;
    }

    write_optional_type_metadata(writer, metadata.return_type_declaration_metadata.as_ref(), session)?;
    write_optional_type_metadata(writer, metadata.return_type_metadata.as_ref(), session)?;
    write_templates(writer, &metadata.template_types, None, None)?;
    write_attributes(writer, &metadata.attributes, session)?;
    writer.write_u32(metadata.thrown_types.len() as u32);
    for thrown in &metadata.thrown_types {
        write_type_metadata(writer, thrown, session)?;
    }

    write_words(writer, metadata.globals_accessed.iter().copied())?;
    write_assertions(writer, &metadata.assertions)?;
    write_assertions(writer, &metadata.if_true_assertions)?;
    write_assertions(writer, &metadata.if_false_assertions)?;
    writer.write_bool(metadata.assertions_inferred);
    writer.write_bool(metadata.has_docblock);
    writer.write_u64(metadata.flags.bits());
    write_version_constraint(writer, &metadata.version_constraint);
    writer.write_bool(metadata.method_metadata.is_some());
    if let Some(method) = &metadata.method_metadata {
        write_visibility(writer, method.visibility);
        writer.write_bool(method.is_final);
        writer.write_bool(method.is_abstract);
        writer.write_bool(method.is_static);
        writer.write_bool(method.is_constructor);
        let mut constraints: Vec<_> = method.where_constraints.iter().collect();
        constraints.sort_unstable_by(|(left, _), (right, _)| left.as_bytes().cmp(right.as_bytes()));
        writer.write_u32(constraints.len() as u32);
        for (name, constraint) in constraints {
            writer.write_bytes(name.as_bytes())?;
            write_type_metadata(writer, constraint, session)?;
        }
    }

    Ok(())
}

fn write_assertions(
    writer: &mut PayloadWriter,
    assertions: &BTreeMap<Word, Vec<Assertion>>,
) -> Result<(), ExternalAnalyzerError> {
    writer.write_u32(assertions.len() as u32);
    for (variable, values) in assertions {
        writer.write_bytes(variable.as_bytes())?;
        writer.write_u32(values.len() as u32);
        for assertion in values {
            write_assertion(writer, assertion)?;
        }
    }
    Ok(())
}

#[allow(clippy::too_many_lines)]
fn write_assertion(writer: &mut PayloadWriter, assertion: &Assertion) -> Result<(), ExternalAnalyzerError> {
    let kind = match assertion {
        Assertion::Any => 1,
        Assertion::IsType(_) => 2,
        Assertion::IsNotType(_) => 3,
        Assertion::Falsy => 4,
        Assertion::Truthy => 5,
        Assertion::IsIdentical(_) => 6,
        Assertion::IsNotIdentical(_) => 7,
        Assertion::IsEqual(_) => 8,
        Assertion::IsNotEqual(_) => 9,
        Assertion::IsEqualIsset => 10,
        Assertion::IsIsset => 11,
        Assertion::IsNotIsset => 12,
        Assertion::HasStringArrayAccess => 13,
        Assertion::HasIntOrStringArrayAccess => 14,
        Assertion::ArrayKeyExists => 15,
        Assertion::ArrayKeyDoesNotExist => 16,
        Assertion::InArray(_) => 17,
        Assertion::NotInArray(_) => 18,
        Assertion::HasArrayKey(_) => 19,
        Assertion::DoesNotHaveArrayKey(_) => 20,
        Assertion::HasNonnullEntryForKey(_) => 21,
        Assertion::DoesNotHaveNonnullEntryForKey(_) => 22,
        Assertion::Empty => 23,
        Assertion::NonEmpty => 24,
        Assertion::NonEmptyCountable(_) => 25,
        Assertion::EmptyCountable => 26,
        Assertion::HasExactCount(_) => 27,
        Assertion::HasAtLeastCount(_) => 28,
        Assertion::DoesNotHaveExactCount(_) => 29,
        Assertion::DoesNotHasAtLeastCount(_) => 30,
        Assertion::IsLessThan(_) => 31,
        Assertion::IsLessThanOrEqual(_) => 32,
        Assertion::IsGreaterThan(_) => 33,
        Assertion::IsGreaterThanOrEqual(_) => 34,
        Assertion::IsLessThanFromBound(_) => 35,
        Assertion::IsLessThanOrEqualFromBound(_) => 36,
        Assertion::IsGreaterThanFromBound(_) => 37,
        Assertion::IsGreaterThanOrEqualFromBound(_) => 38,
        Assertion::IsLessThanVariable(_) => 39,
        Assertion::IsLessThanOrEqualVariable(_) => 40,
        Assertion::IsGreaterThanVariable(_) => 41,
        Assertion::IsGreaterThanOrEqualVariable(_) => 42,
        Assertion::Countable => 43,
        Assertion::NotCountable(_) => 44,
    };
    writer.write_u8(kind);

    match assertion {
        Assertion::IsType(atomic)
        | Assertion::IsNotType(atomic)
        | Assertion::IsIdentical(atomic)
        | Assertion::IsNotIdentical(atomic)
        | Assertion::IsEqual(atomic)
        | Assertion::IsNotEqual(atomic) => write_atomic(writer, atomic)?,
        Assertion::InArray(union) | Assertion::NotInArray(union) => write_union(writer, union)?,
        Assertion::HasArrayKey(key)
        | Assertion::DoesNotHaveArrayKey(key)
        | Assertion::HasNonnullEntryForKey(key)
        | Assertion::DoesNotHaveNonnullEntryForKey(key) => write_array_key(writer, key)?,
        Assertion::NonEmptyCountable(negatable) | Assertion::NotCountable(negatable) => {
            writer.write_bool(*negatable);
        }
        Assertion::HasExactCount(value)
        | Assertion::HasAtLeastCount(value)
        | Assertion::DoesNotHaveExactCount(value)
        | Assertion::DoesNotHasAtLeastCount(value) => writer.write_u64(*value as u64),
        Assertion::IsLessThan(value)
        | Assertion::IsLessThanOrEqual(value)
        | Assertion::IsGreaterThan(value)
        | Assertion::IsGreaterThanOrEqual(value)
        | Assertion::IsLessThanFromBound(value)
        | Assertion::IsLessThanOrEqualFromBound(value)
        | Assertion::IsGreaterThanFromBound(value)
        | Assertion::IsGreaterThanOrEqualFromBound(value) => writer.write_u64(*value as u64),
        Assertion::IsLessThanVariable(variable)
        | Assertion::IsLessThanOrEqualVariable(variable)
        | Assertion::IsGreaterThanVariable(variable)
        | Assertion::IsGreaterThanOrEqualVariable(variable) => writer.write_bytes(variable.as_bytes())?,
        _ => {}
    }

    Ok(())
}

fn write_atomic(writer: &mut PayloadWriter, atomic: &TAtomic) -> Result<(), ExternalAnalyzerError> {
    write_union(writer, &TUnion::from_atomic(atomic.clone()))
}

fn write_array_key(writer: &mut PayloadWriter, key: &ArrayKey) -> Result<(), ExternalAnalyzerError> {
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

fn write_parameter(
    writer: &mut PayloadWriter,
    metadata: &FunctionLikeParameterMetadata,
    session: &ExternalAnalysisSession,
) -> Result<(), ExternalAnalyzerError> {
    writer.write_bytes(metadata.name.0.as_bytes())?;
    write_location(writer, metadata.span, session)?;
    write_location(writer, metadata.name_span, session)?;
    write_optional_type_metadata(writer, metadata.type_declaration_metadata.as_ref(), session)?;
    write_optional_type_metadata(writer, metadata.type_metadata.as_ref(), session)?;
    write_optional_type_metadata(writer, metadata.out_type.as_ref(), session)?;
    write_optional_type_metadata(writer, metadata.closure_this_type.as_ref(), session)?;
    write_optional_type_metadata(writer, metadata.default_type.as_ref(), session)?;
    write_attributes(writer, &metadata.attributes, session)?;
    writer.write_u64(metadata.flags.bits());
    Ok(())
}

pub(super) fn write_property(
    writer: &mut PayloadWriter,
    metadata: &PropertyMetadata,
    session: &ExternalAnalysisSession,
) -> Result<(), ExternalAnalyzerError> {
    writer.write_bytes(metadata.name.0.as_bytes())?;
    write_optional_location(writer, metadata.span, session)?;
    write_optional_location(writer, metadata.name_span, session)?;
    write_visibility(writer, metadata.read_visibility);
    write_visibility(writer, metadata.write_visibility);
    write_optional_type_metadata(writer, metadata.type_declaration_metadata.as_ref(), session)?;
    write_optional_type_metadata(writer, metadata.type_metadata.as_ref(), session)?;
    write_optional_type_metadata(writer, metadata.write_type_metadata.as_ref(), session)?;
    write_optional_type_metadata(writer, metadata.default_type_metadata.as_ref(), session)?;
    writer.write_u64(metadata.flags.bits());
    write_property_hooks(writer, &metadata.hooks, session)?;
    write_version_constraint(writer, &metadata.version_constraint);
    Ok(())
}

fn write_property_hooks(
    writer: &mut PayloadWriter,
    hooks: &mago_word::WordMap<PropertyHookMetadata>,
    session: &ExternalAnalysisSession,
) -> Result<(), ExternalAnalyzerError> {
    let mut hooks: Vec<_> = hooks.values().collect();
    hooks.sort_unstable_by(|left, right| left.name.as_bytes().cmp(right.name.as_bytes()));
    writer.write_u32(hooks.len() as u32);
    for hook in hooks {
        writer.write_bytes(hook.name.as_bytes())?;
        write_location(writer, hook.span, session)?;
        writer.write_u64(hook.flags.bits());
        writer.write_bool(hook.parameter.is_some());
        if let Some(parameter) = &hook.parameter {
            write_parameter(writer, parameter, session)?;
        }

        writer.write_bool(hook.returns_by_ref);
        writer.write_bool(hook.is_abstract);
        write_attributes(writer, &hook.attributes, session)?;
        write_optional_type_metadata(writer, hook.return_type_metadata.as_ref(), session)?;
        writer.write_bool(hook.has_docblock);
    }

    Ok(())
}

fn write_class_constant(
    writer: &mut PayloadWriter,
    metadata: &ClassLikeConstantMetadata,
    session: &ExternalAnalysisSession,
) -> Result<(), ExternalAnalyzerError> {
    writer.write_bytes(metadata.name.as_bytes())?;
    write_location(writer, metadata.span, session)?;
    write_visibility(writer, metadata.visibility);
    write_optional_type_metadata(writer, metadata.type_declaration.as_ref(), session)?;
    write_optional_type_metadata(writer, metadata.type_metadata.as_ref(), session)?;
    write_optional_atomic(writer, metadata.inferred_type.as_ref())?;
    write_attributes(writer, &metadata.attributes, session)?;
    writer.write_u64(metadata.flags.bits());
    write_version_constraint(writer, &metadata.version_constraint);
    Ok(())
}

fn write_enum_case(
    writer: &mut PayloadWriter,
    metadata: &EnumCaseMetadata,
    session: &ExternalAnalysisSession,
) -> Result<(), ExternalAnalyzerError> {
    writer.write_bytes(metadata.name.as_bytes())?;
    write_location(writer, metadata.span, session)?;
    write_location(writer, metadata.name_span, session)?;
    write_optional_atomic(writer, metadata.value_type.as_ref())?;
    write_attributes(writer, &metadata.attributes, session)?;
    writer.write_u64(metadata.flags.bits());
    write_version_constraint(writer, &metadata.version_constraint);
    Ok(())
}

fn write_constant(
    writer: &mut PayloadWriter,
    metadata: &ConstantMetadata,
    session: &ExternalAnalysisSession,
) -> Result<(), ExternalAnalyzerError> {
    writer.write_bytes(metadata.name.as_bytes())?;
    write_location(writer, metadata.span, session)?;
    write_optional_type_metadata(writer, metadata.type_metadata.as_ref(), session)?;
    write_optional_union(writer, metadata.inferred_type.as_ref())?;
    write_attributes(writer, &metadata.attributes, session)?;
    writer.write_u64(metadata.flags.bits());
    write_version_constraint(writer, &metadata.version_constraint);
    Ok(())
}

fn write_location(
    writer: &mut PayloadWriter,
    span: Span,
    session: &ExternalAnalysisSession,
) -> Result<(), ExternalAnalyzerError> {
    let source = session.source_name(span.file_id);
    writer.write_bool(source.is_some());
    if let Some(source) = source {
        writer.write_bytes(source)?;
    }

    writer.write_u32(span.start.offset);
    writer.write_u32(span.end.offset);
    Ok(())
}

fn write_optional_location(
    writer: &mut PayloadWriter,
    span: Option<Span>,
    session: &ExternalAnalysisSession,
) -> Result<(), ExternalAnalyzerError> {
    writer.write_bool(span.is_some());
    if let Some(span) = span {
        write_location(writer, span, session)?;
    }

    Ok(())
}

fn write_type_metadata(
    writer: &mut PayloadWriter,
    metadata: &TypeMetadata,
    session: &ExternalAnalysisSession,
) -> Result<(), ExternalAnalyzerError> {
    write_location(writer, metadata.span, session)?;
    write_union(writer, &metadata.type_union)?;
    writer.write_bool(metadata.from_docblock);
    writer.write_bool(metadata.inferred);
    Ok(())
}

fn write_optional_type_metadata(
    writer: &mut PayloadWriter,
    metadata: Option<&TypeMetadata>,
    session: &ExternalAnalysisSession,
) -> Result<(), ExternalAnalyzerError> {
    writer.write_bool(metadata.is_some());
    if let Some(metadata) = metadata {
        write_type_metadata(writer, metadata, session)?;
    }

    Ok(())
}

fn write_union(writer: &mut PayloadWriter, union: &TUnion) -> Result<(), ExternalAnalyzerError> {
    let mut references = Vec::new();
    encode_union_snapshot(writer, union, &mut references, 0)
}

trait MixinType {
    fn type_union(&self) -> &TUnion;
}

impl MixinType for TUnion {
    #[inline]
    fn type_union(&self) -> &TUnion {
        self
    }
}

impl MixinType for TypeMetadata {
    #[inline]
    fn type_union(&self) -> &TUnion {
        &self.type_union
    }
}

fn write_mixins<T>(writer: &mut PayloadWriter, mixins: &[T]) -> Result<(), ExternalAnalyzerError>
where
    T: MixinType,
{
    writer.write_u32(mixins.len() as u32);
    for mixin in mixins {
        write_union(writer, mixin.type_union())?;
    }

    Ok(())
}

fn write_optional_union(writer: &mut PayloadWriter, union: Option<&TUnion>) -> Result<(), ExternalAnalyzerError> {
    writer.write_bool(union.is_some());
    if let Some(union) = union {
        write_union(writer, union)?;
    }

    Ok(())
}

fn write_optional_atomic(writer: &mut PayloadWriter, atomic: Option<&TAtomic>) -> Result<(), ExternalAnalyzerError> {
    writer.write_bool(atomic.is_some());
    if let Some(atomic) = atomic {
        write_union(writer, &TUnion::from_atomic(atomic.clone()))?;
    }

    Ok(())
}

fn write_attributes(
    writer: &mut PayloadWriter,
    attributes: &[AttributeMetadata],
    session: &ExternalAnalysisSession,
) -> Result<(), ExternalAnalyzerError> {
    writer.write_u32(attributes.len() as u32);
    for attribute in attributes {
        writer.write_bytes(attribute.name.as_bytes())?;
        write_location(writer, attribute.span, session)?;
    }

    Ok(())
}

fn write_templates(
    writer: &mut PayloadWriter,
    templates: &TemplateTypes,
    variances: Option<&[Variance]>,
    readonly: Option<&mago_word::WordSet>,
) -> Result<(), ExternalAnalyzerError> {
    writer.write_u32(templates.len() as u32);
    for (index, (name, template)) in templates.iter().enumerate() {
        writer.write_bytes(name.as_bytes())?;
        encode_generic_parent(writer, template.defining_entity)?;
        write_union(writer, &template.constraint)?;
        write_optional_union(writer, template.default.as_ref())?;
        writer
            .write_u8(variance(variances.and_then(|values| values.get(index)).copied().unwrap_or(Variance::Invariant)));
        writer.write_bool(readonly.is_some_and(|names| names.contains(name)));
    }

    Ok(())
}

fn write_type_aliases(
    writer: &mut PayloadWriter,
    aliases: &mago_word::WordMap<TypeMetadata>,
    session: &ExternalAnalysisSession,
) -> Result<(), ExternalAnalyzerError> {
    let mut aliases: Vec<_> = aliases.iter().collect();
    aliases.sort_unstable_by(|(left, _), (right, _)| left.as_bytes().cmp(right.as_bytes()));
    writer.write_u32(aliases.len() as u32);
    for (name, metadata) in aliases {
        writer.write_bytes(name.as_bytes())?;
        write_type_metadata(writer, metadata, session)?;
    }

    Ok(())
}

fn write_words(writer: &mut PayloadWriter, words: impl IntoIterator<Item = Word>) -> Result<(), ExternalAnalyzerError> {
    let mut words: Vec<_> = words.into_iter().collect();
    words.sort_unstable_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    writer.write_u32(words.len() as u32);
    for word in words {
        writer.write_bytes(word.as_bytes())?;
    }

    Ok(())
}

fn write_optional_words<I>(writer: &mut PayloadWriter, words: Option<I>) -> Result<(), ExternalAnalyzerError>
where
    I: IntoIterator<Item = Word>,
{
    writer.write_bool(words.is_some());
    if let Some(words) = words {
        write_words(writer, words)?;
    }

    Ok(())
}

fn write_optional_word(writer: &mut PayloadWriter, word: Option<Word>) -> Result<(), ExternalAnalyzerError> {
    writer.write_bool(word.is_some());
    if let Some(word) = word {
        writer.write_bytes(word.as_bytes())?;
    }

    Ok(())
}

fn write_version_constraint(writer: &mut PayloadWriter, constraint: &VersionConstraint) {
    writer.write_u32(constraint.ranges.len() as u32);
    for range in &constraint.ranges {
        writer.write_bool(range.min.is_some());
        if let Some(minimum) = range.min {
            writer.write_u32(minimum.to_version_id());
        }

        writer.write_bool(range.max.is_some());
        if let Some(maximum) = range.max {
            writer.write_u32(maximum.to_version_id());
        }
    }
}

fn write_optional_bool(writer: &mut PayloadWriter, value: Option<bool>) {
    writer.write_u8(match value {
        None => 0,
        Some(false) => 1,
        Some(true) => 2,
    });
}

fn write_visibility(writer: &mut PayloadWriter, visibility: Visibility) {
    writer.write_u8(match visibility {
        Visibility::Public => 1,
        Visibility::Protected => 2,
        Visibility::Private => 3,
    });
}

fn symbol_kind(kind: SymbolKind) -> u8 {
    match kind {
        SymbolKind::Class => 1,
        SymbolKind::Enum => 2,
        SymbolKind::Trait => 3,
        SymbolKind::Interface => 4,
    }
}

fn class_like_matches(kind: SymbolKind, filter: u8) -> bool {
    match filter {
        ANY_CLASS_LIKE => true,
        CLASS => kind == SymbolKind::Class,
        INTERFACE => kind == SymbolKind::Interface,
        TRAIT => kind == SymbolKind::Trait,
        ENUM => kind == SymbolKind::Enum,
        _ => false,
    }
}

fn variance(value: Variance) -> u8 {
    match value {
        Variance::Invariant => 1,
        Variance::Covariant => 2,
        Variance::Contravariant => 3,
        Variance::Bivariant => 4,
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use crate::external::protocol;
    use crate::external::protocol::NestedRequestKind;

    fn session(generation: u64) -> ExternalAnalysisSession {
        ExternalAnalysisSession { generation, sources: foldhash::HashMap::default() }
    }

    #[test]
    fn batched_existence_query_round_trips_in_request_order() {
        let session = session(17);
        let codebase = CodebaseMetadata::new();
        let mut writer = protocol::message_writer(CODEBASE_QUERY_REQUEST);
        writer.write_u64(17);
        writer.write_u8(CHECK_EXISTENCE);
        writer.write_u8(EXISTS_CLASS_LIKE);
        writer.write_u32(2);
        writer.write_bytes(b"MissingA").expect("name should fit in a frame");
        writer.write_bytes(b"MissingB").expect("name should fit in a frame");

        let (kind, response) = protocol::handle_nested_request(&writer.finish(), &codebase, &session, |_| None)
            .expect("metadata query should succeed");
        assert_eq!(kind, NestedRequestKind::CodebaseQuery);

        let mut reader = protocol::message_reader(&response, CODEBASE_QUERY_RESPONSE)
            .expect("metadata response should have a valid header");
        assert_eq!(reader.read_u64("generation").expect("generation should decode"), 17);
        assert_eq!(reader.read_u8("operation").expect("operation should decode"), CHECK_EXISTENCE);
        assert_eq!(reader.read_u8("predicate").expect("predicate should decode"), EXISTS_CLASS_LIKE);
        assert_eq!(reader.read_u32("count").expect("count should decode"), 2);
        assert!(!reader.read_bool("first result").expect("first result should decode"));
        assert!(!reader.read_bool("second result").expect("second result should decode"));
        reader.finish().expect("metadata response should contain no trailing bytes");
    }

    #[test]
    fn stale_generation_is_rejected_before_query_dispatch() {
        let session = session(19);
        let codebase = CodebaseMetadata::new();
        let mut writer = protocol::message_writer(CODEBASE_QUERY_REQUEST);
        writer.write_u64(18);

        let error = protocol::handle_nested_request(&writer.finish(), &codebase, &session, |_| None)
            .expect_err("stale metadata queries must fail");
        assert!(error.to_string().contains("active generation is 19"));
    }
}
