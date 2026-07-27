//! `EntityRepository::findBy()` / `findOneBy()` / `count()` criteria-key validator.
//!
//! Validates the string keys of a literal criteria array against the persisted
//! fields of the entity managed by the receiving repository. The entity is
//! resolved from the repository's generic parameter (`EntityRepository<T>`),
//! and the persisted-field map is built from the ORM mapping attributes
//! (`#[ORM\Column]`, `#[ORM\Id]`, associations, `#[ORM\Embedded]`) recorded in
//! the codex. Attribute mapping only; entities without any attribute-mapped
//! property produce no opinion, so XML/YAML mappings never cause false
//! positives.

use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::named::TNamedObject;
use mago_codex::ttype::expander::StaticClassType;
use mago_codex::ttype::union::TUnion;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::cst::ArrayElement;
use mago_syntax::cst::Expression;
use mago_word::Word;
use mago_word::ascii_lowercase_word;

use crate::code::IssueCode;
use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::method::MethodReturnTypeProvider;
use crate::plugin::provider::method::MethodTarget;

static META: ProviderMeta = ProviderMeta::new(
    "doctrine::repository::criteria",
    "EntityRepository criteria keys",
    "Validates criteria keys passed to repository finder methods against the entity's persisted fields",
);

// Use wildcards for the class since repositories are arbitrary subclasses of
// `EntityRepository`; the receiver is checked with `is_instance_of` below.
static TARGETS: [MethodTarget; 3] =
    [MethodTarget::any_class(b"findBy"), MethodTarget::any_class(b"findOneBy"), MethodTarget::any_class(b"count")];

/// Repository base types whose first template parameter is the managed entity.
const REPOSITORY_BASES: [&[u8]; 2] = [b"Doctrine\\ORM\\EntityRepository", b"Doctrine\\Persistence\\ObjectRepository"];

/// Property attributes that mark a property as persisted by Doctrine ORM.
const PERSISTED_FIELD_ATTRIBUTES: [&[u8]; 7] = [
    b"Doctrine\\ORM\\Mapping\\Column",
    b"Doctrine\\ORM\\Mapping\\Id",
    b"Doctrine\\ORM\\Mapping\\ManyToOne",
    b"Doctrine\\ORM\\Mapping\\OneToOne",
    b"Doctrine\\ORM\\Mapping\\OneToMany",
    b"Doctrine\\ORM\\Mapping\\ManyToMany",
    b"Doctrine\\ORM\\Mapping\\Embedded",
];

/// Validator for criteria keys passed to Doctrine repository finder methods.
///
/// Reports `doctrine-unknown-field` when a literal string key does not match
/// any persisted field or association of the managed entity, with the closest
/// matching field name as a suggestion. Never provides a return type: normal
/// analysis (driven by doctrine/orm's own docblock generics) always proceeds.
#[derive(Default)]
pub struct FindByFieldValidator;

impl Provider for FindByFieldValidator {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl MethodReturnTypeProvider for FindByFieldValidator {
    fn targets() -> &'static [MethodTarget] {
        &TARGETS
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        class_name: &[u8],
        _method_name: &[u8],
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        if !REPOSITORY_BASES.iter().any(|base| context.is_instance_of(class_name, base)) {
            return None;
        }

        let criteria = invocation.get_argument(0, &[b"criteria"])?;
        let elements = criteria.get_array_like_elements()?;

        let entity_metadata = resolve_entity_metadata(context, invocation)?;
        let fields = persisted_fields(context, entity_metadata);
        if fields.is_empty() {
            // No attribute-mapped property: either not an entity, or mapped
            // through XML/YAML. No opinion, never a false positive.
            return None;
        }

        for element in elements {
            let ArrayElement::KeyValue(key_value) = element else {
                continue;
            };

            let Some(key) =
                context.get_expression_type(key_value.key).and_then(TUnion::get_single_literal_string_value)
            else {
                continue;
            };

            // Nested paths (e.g. `association.field`) are handled by some
            // repository extensions; stay out of them.
            if key.is_empty() || key.contains(&b'.') {
                continue;
            }

            if fields.iter().any(|field| field_name(field) == key) {
                continue;
            }

            report_unknown_field(context, entity_metadata, &fields, key, key_value.key);
        }

        None
    }
}

/// Resolves the metadata of the entity managed by the receiving repository.
///
/// The entity is the first template parameter of the repository base type,
/// read either directly from the receiver's type parameters
/// (`EntityRepository<User>`) or through the receiver class's inheritance
/// (`UserRepository extends ServiceEntityRepository<User>`).
fn resolve_entity_metadata<'codebase>(
    context: &ProviderContext<'codebase, '_, '_>,
    invocation: &InvocationInfo<'_, '_, '_>,
) -> Option<&'codebase ClassLikeMetadata> {
    let method_context = invocation.inner().target.get_method_context()?;
    let StaticClassType::Object(TObject::Named(receiver)) = &method_context.class_type else {
        return None;
    };

    let entity_name = resolve_entity_name(context, receiver)?;

    context.codebase().get_class_like(entity_name.as_bytes())
}

/// Resolves the name of the entity from the receiver's object type.
fn resolve_entity_name(context: &ProviderContext<'_, '_, '_>, receiver: &TNamedObject) -> Option<Word> {
    let receiver_name = ascii_lowercase_word(receiver.get_name().as_bytes());

    for base in REPOSITORY_BASES {
        if receiver_name != ascii_lowercase_word(base) {
            continue;
        }

        // The receiver is the repository base itself: `EntityRepository<T>`.
        return single_object_name(receiver.get_type_parameters()?.first()?);
    }

    let receiver_metadata = context.codebase().get_class_like(receiver.get_name().as_bytes())?;

    for base in REPOSITORY_BASES {
        let base_metadata = context.codebase().get_class_like(base)?;
        let template_name = base_metadata.get_template_name_for_index(0)?;

        let Some(extended_parameters) = receiver_metadata.template_extended_parameters.get(&base_metadata.name) else {
            continue;
        };

        let entity = extended_parameters.get(&template_name)?;
        if let Some(name) = single_object_name(entity) {
            return Some(name);
        }

        // The receiver class is itself generic (e.g. a use site typed
        // `ServiceEntityRepository<User>`): hop through its own parameter.
        if entity.is_single()
            && let TAtomic::GenericParameter(parameter) = entity.get_single()
        {
            let index = receiver_metadata.get_template_index_for_name(parameter.parameter_name)?;

            return single_object_name(receiver.get_type_parameters()?.get(index)?);
        }

        return None;
    }

    None
}

/// Returns the class name of a union that is a single named object.
fn single_object_name(union: &TUnion) -> Option<Word> {
    union.get_single_named_object().map(TNamedObject::get_name)
}

/// Collects the names of the entity's persisted properties: every non-static
/// property (own or inherited, e.g. from a mapped superclass) carrying one of
/// the ORM mapping attributes.
fn persisted_fields(context: &ProviderContext<'_, '_, '_>, entity_metadata: &ClassLikeMetadata) -> Vec<Word> {
    let mut fields: Vec<Word> = Vec::new();

    let parents = entity_metadata.all_parent_classes.iter().copied();
    for class_name in std::iter::once(entity_metadata.name).chain(parents) {
        let Some(class_metadata) = context.codebase().get_class_like(class_name.as_bytes()) else {
            continue;
        };

        for (property_name, property) in &class_metadata.properties {
            if property.flags.is_static() || fields.contains(property_name) {
                continue;
            }

            let is_persisted = property.attributes.iter().any(|attribute| {
                PERSISTED_FIELD_ATTRIBUTES.iter().any(|orm| attribute.name.as_bytes().eq_ignore_ascii_case(orm))
            });

            if is_persisted {
                fields.push(*property_name);
            }
        }
    }

    fields
}

/// Strips the leading `$` from a property name to get the field name.
fn field_name(property_name: &Word) -> &[u8] {
    property_name.as_bytes().strip_prefix(b"$").unwrap_or(property_name.as_bytes())
}

fn report_unknown_field(
    context: &ProviderContext<'_, '_, '_>,
    entity_metadata: &ClassLikeMetadata,
    fields: &[Word],
    key: &[u8],
    key_expression: &Expression<'_>,
) {
    let entity_name = entity_metadata.original_name;
    let key_display = String::from_utf8_lossy(key);

    let mut issue = Issue::error(format!(
        "Entity `{entity_name}` has no persisted field or association named `{key_display}`."
    ))
    .with_annotation(
        Annotation::primary(key_expression.span())
            .with_message(format!("Unknown field `{key_display}` used as a criteria key")),
    )
    .with_note(format!(
        "Persisted fields are read from the ORM mapping attributes (`#[ORM\\Column]`, `#[ORM\\Id]`, associations, `#[ORM\\Embedded]`) declared on `{entity_name}`."
    ));

    if let Some(suggestion) = closest_field(fields, key) {
        let suggestion_display = String::from_utf8_lossy(suggestion);

        issue = issue.with_help(format!("Did you mean `{suggestion_display}`?"));
    }

    context.report(IssueCode::DoctrineUnknownField, issue);
}

/// Finds the persisted field closest to `key`, within a small edit distance.
fn closest_field<'fields>(fields: &'fields [Word], key: &[u8]) -> Option<&'fields [u8]> {
    const MAX_DISTANCE: usize = 3;

    fields
        .iter()
        .map(field_name)
        .map(|name| (levenshtein_distance(key, name), name))
        .filter(|(distance, _)| *distance <= MAX_DISTANCE)
        .min_by_key(|(distance, _)| *distance)
        .map(|(_, name)| name)
}

/// Computes the Levenshtein distance between two byte strings.
fn levenshtein_distance(a: &[u8], b: &[u8]) -> usize {
    if a == b {
        return 0;
    }

    if a.is_empty() {
        return b.len();
    }

    if b.is_empty() {
        return a.len();
    }

    let mut row: Vec<usize> = (0..=b.len()).collect();

    for (i, byte_a) in a.iter().enumerate() {
        let mut previous_diagonal = row[0];
        row[0] = i + 1;

        for (j, byte_b) in b.iter().enumerate() {
            let previous = row[j + 1];
            let substitution = previous_diagonal + usize::from(byte_a != byte_b);

            row[j + 1] = substitution.min(previous + 1).min(row[j] + 1);
            previous_diagonal = previous;
        }
    }

    row[b.len()]
}
