use foldhash::HashSet;

use mago_codex::metadata::CodebaseMetadata;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::metadata::class_like_constant::ClassLikeConstantMetadata;
use mago_codex::metadata::constant::ConstantMetadata;
use mago_codex::metadata::enum_case::EnumCaseMetadata;
use mago_codex::metadata::flags::MetadataFlags;
use mago_codex::metadata::function_like::FunctionLikeKind;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_codex::metadata::parameter::FunctionLikeParameterMetadata;
use mago_codex::metadata::property::PropertyMetadata;
use mago_codex::metadata::ttype::TypeMetadata;
use mago_codex::misc::GenericParent;
use mago_codex::misc::VariableIdentifier;
use mago_codex::symbol::SymbolKind;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::template::GenericTemplate;
use mago_codex::ttype::template::variance::Variance;
use mago_codex::ttype::union::TUnion;
use mago_codex::visibility::Visibility;
use mago_extension::PayloadReader;
use mago_extension::PayloadWriter;
use mago_span::Span;
use mago_word::Word;
use mago_word::ascii_lowercase_constant_name_word;
use mago_word::ascii_lowercase_word;
use mago_word::empty_word;
use mago_word::word;

use crate::external::ExternalAnalysisSession;
use crate::external::error::ExternalAnalyzerError;
use crate::external::error::protocol;
use crate::external::protocol;
use crate::external::protocol::NestedRequestKind;

pub(super) const CODEBASE_MUTATION_REQUEST: u16 = 10;
const CODEBASE_MUTATION_RESPONSE: u16 = 0x800A;

const REMOVE_CLASS_LIKES: u8 = 1;
const INSERT_CLASS_LIKES: u8 = 2;
const REMOVE_FUNCTIONS: u8 = 3;
const INSERT_FUNCTIONS: u8 = 4;
const REMOVE_CONSTANTS: u8 = 5;
const INSERT_CONSTANTS: u8 = 6;

const MAXIMUM_MUTATIONS: usize = 0x0001_0000;

pub(super) fn is_request(payload: &[u8]) -> Result<bool, ExternalAnalyzerError> {
    Ok(protocol::message_kind(payload)? == CODEBASE_MUTATION_REQUEST)
}

pub(super) fn handle_request(
    payload: &[u8],
    codebase: &mut CodebaseMetadata,
    session: &ExternalAnalysisSession,
) -> Result<(NestedRequestKind, Vec<u8>), ExternalAnalyzerError> {
    let mut reader = protocol::message_reader(payload, CODEBASE_MUTATION_REQUEST)?;
    let generation = reader.read_u64("codebase mutation generation")?;
    if generation != session.generation() {
        return Err(protocol(format!(
            "metadata mutation targets generation {generation}, but the active generation is {}",
            session.generation()
        )));
    }

    let operation = reader.read_u8("codebase mutation operation")?;
    let mut writer = protocol::message_writer(CODEBASE_MUTATION_RESPONSE);
    writer.write_u64(generation);
    writer.write_u8(operation);
    match operation {
        REMOVE_CLASS_LIKES => remove_class_likes(&mut reader, &mut writer, codebase)?,
        INSERT_CLASS_LIKES => insert_class_likes(&mut reader, &mut writer, codebase)?,
        REMOVE_FUNCTIONS => remove_functions(&mut reader, &mut writer, codebase)?,
        INSERT_FUNCTIONS => insert_functions(&mut reader, &mut writer, codebase)?,
        REMOVE_CONSTANTS => remove_constants(&mut reader, &mut writer, codebase)?,
        INSERT_CONSTANTS => insert_constants(&mut reader, &mut writer, codebase)?,
        unknown => return Err(protocol(format!("unknown codebase mutation operation {unknown}"))),
    }
    reader.finish()?;

    Ok((NestedRequestKind::CodebaseMutation, writer.finish()))
}

fn remove_class_likes(
    reader: &mut PayloadReader<'_>,
    writer: &mut PayloadWriter,
    codebase: &mut CodebaseMetadata,
) -> Result<(), ExternalAnalyzerError> {
    let names = read_names(reader, "class-like removals")?;
    writer.write_u32(names.len() as u32);
    for name in names {
        let lookup = ascii_lowercase_word(name.as_bytes());
        let removed = codebase.class_likes.remove(&lookup);
        writer.write_bool(removed.is_some());
        if let Some(metadata) = removed {
            write_class_like_definition(writer, &metadata, codebase)?;
            codebase.function_likes.retain(|(scope, _), _| *scope != lookup);
            codebase.symbols.remove(lookup);
        }
    }

    Ok(())
}

fn insert_class_likes(
    reader: &mut PayloadReader<'_>,
    writer: &mut PayloadWriter,
    codebase: &mut CodebaseMetadata,
) -> Result<(), ExternalAnalyzerError> {
    let count = reader.read_count("class-like insertions", MAXIMUM_MUTATIONS)?;
    let mut definitions = Vec::with_capacity(count);
    let mut names = HashSet::default();
    for _ in 0..count {
        let (metadata, methods) = read_class_like_definition(reader)?;
        if !names.insert(metadata.name) {
            return Err(protocol(format!("class-like insertion contains duplicate `{}`", metadata.original_name)));
        }
        if codebase.class_likes.contains_key(&metadata.name) {
            return Err(protocol(format!("class-like `{}` already exists", metadata.original_name)));
        }
        definitions.push((metadata, methods));
    }

    for (metadata, methods) in definitions {
        let name = metadata.name;
        match metadata.kind {
            SymbolKind::Class => codebase.symbols.add_class_name(name),
            SymbolKind::Interface => codebase.symbols.add_interface_name(name),
            SymbolKind::Trait => codebase.symbols.add_trait_name(name),
            SymbolKind::Enum => codebase.symbols.add_enum_name(name),
        }
        for method in methods {
            codebase.function_likes.insert((name, method.name), method);
        }
        codebase.class_likes.insert(name, metadata);
    }
    writer.write_u32(count as u32);
    Ok(())
}

fn remove_functions(
    reader: &mut PayloadReader<'_>,
    writer: &mut PayloadWriter,
    codebase: &mut CodebaseMetadata,
) -> Result<(), ExternalAnalyzerError> {
    let names = read_names(reader, "function removals")?;
    writer.write_u32(names.len() as u32);
    for name in names {
        let lookup = ascii_lowercase_word(name.as_bytes());
        let removed = codebase.function_likes.remove(&(empty_word(), lookup));
        writer.write_bool(removed.is_some());
        if let Some(metadata) = removed {
            write_function_definition(writer, &metadata)?;
        }
    }
    Ok(())
}

fn insert_functions(
    reader: &mut PayloadReader<'_>,
    writer: &mut PayloadWriter,
    codebase: &mut CodebaseMetadata,
) -> Result<(), ExternalAnalyzerError> {
    let count = reader.read_count("function insertions", MAXIMUM_MUTATIONS)?;
    let mut definitions = Vec::with_capacity(count);
    let mut names = HashSet::default();
    for _ in 0..count {
        let metadata = read_function_definition(reader, FunctionLikeKind::Function, None)?;
        if !names.insert(metadata.name) {
            return Err(protocol(format!("function insertion contains duplicate `{}`", metadata.original_name)));
        }
        if codebase.function_likes.contains_key(&(empty_word(), metadata.name)) {
            return Err(protocol(format!("function `{}` already exists", metadata.original_name)));
        }
        definitions.push(metadata);
    }

    for metadata in definitions {
        codebase.function_likes.insert((empty_word(), metadata.name), metadata);
    }
    writer.write_u32(count as u32);
    Ok(())
}

fn remove_constants(
    reader: &mut PayloadReader<'_>,
    writer: &mut PayloadWriter,
    codebase: &mut CodebaseMetadata,
) -> Result<(), ExternalAnalyzerError> {
    let names = read_names(reader, "constant removals")?;
    writer.write_u32(names.len() as u32);
    for name in names {
        let lookup = ascii_lowercase_constant_name_word(name.as_bytes());
        let removed = codebase.constants.remove(&lookup);
        writer.write_bool(removed.is_some());
        if let Some(metadata) = removed {
            write_constant_definition(writer, &metadata)?;
        }
    }
    Ok(())
}

fn insert_constants(
    reader: &mut PayloadReader<'_>,
    writer: &mut PayloadWriter,
    codebase: &mut CodebaseMetadata,
) -> Result<(), ExternalAnalyzerError> {
    let count = reader.read_count("constant insertions", MAXIMUM_MUTATIONS)?;
    let mut definitions = Vec::with_capacity(count);
    let mut names = HashSet::default();
    for _ in 0..count {
        let metadata = read_constant_definition(reader)?;
        let lookup = ascii_lowercase_constant_name_word(metadata.name.as_bytes());
        if !names.insert(lookup) {
            return Err(protocol(format!("constant insertion contains duplicate `{}`", metadata.name)));
        }
        if codebase.constants.contains_key(&lookup) {
            return Err(protocol(format!("constant `{}` already exists", metadata.name)));
        }
        definitions.push((lookup, metadata));
    }

    for (lookup, metadata) in definitions {
        codebase.constants.insert(lookup, metadata);
    }
    writer.write_u32(count as u32);
    Ok(())
}

fn read_names(reader: &mut PayloadReader<'_>, description: &'static str) -> Result<Vec<Word>, ExternalAnalyzerError> {
    let count = reader.read_count(description, MAXIMUM_MUTATIONS)?;
    let mut names = Vec::with_capacity(count);
    for _ in 0..count {
        names.push(read_name(reader, description)?);
    }
    Ok(names)
}

fn read_class_like_definition(
    reader: &mut PayloadReader<'_>,
) -> Result<(ClassLikeMetadata, Vec<FunctionLikeMetadata>), ExternalAnalyzerError> {
    let original_name = read_name(reader, "class-like name")?;
    let name = ascii_lowercase_word(original_name.as_bytes());
    let kind = read_symbol_kind(reader)?;
    let mut metadata = ClassLikeMetadata::new(name, original_name, Span::zero(), None, read_flags(reader)?);
    metadata.kind = kind;
    metadata.direct_parent_class =
        read_optional_name(reader, "parent class")?.map(|name| ascii_lowercase_word(name.as_bytes()));
    metadata.direct_parent_interfaces = read_name_set(reader, "parent interfaces", true)?;
    metadata.require_extends = read_name_set(reader, "required parent classes", true)?;
    metadata.require_implements = read_name_set(reader, "required interfaces", true)?;
    metadata.used_traits = read_name_set(reader, "used traits", true)?;
    metadata.enum_type = read_optional_atomic(reader, "enum backing type")?;
    let templates = read_templates(reader, GenericParent::ClassLike(name))?;
    for (template_name, template, variance, readonly) in templates {
        metadata.template_types.insert(template_name, template);
        metadata.template_variance.push(variance);
        if readonly {
            metadata.template_readonly.insert(template_name);
        }
    }

    let extended_count = reader.read_count("class-like extended types", MAXIMUM_MUTATIONS)?;
    for _ in 0..extended_count {
        let parent = ascii_lowercase_word(read_name(reader, "extended type parent")?.as_bytes());
        let type_count = reader.read_count("extended type arguments", MAXIMUM_MUTATIONS)?;
        let mut types = Vec::with_capacity(type_count);
        for _ in 0..type_count {
            types.push(protocol::decode_complete_union(reader, 0)?);
        }
        metadata.template_extended_offsets.insert(parent, types);
    }

    let alias_count = reader.read_count("class-like type aliases", MAXIMUM_MUTATIONS)?;
    for _ in 0..alias_count {
        let alias = read_name(reader, "type alias name")?;
        metadata
            .type_aliases
            .insert(alias, TypeMetadata::from_docblock(protocol::decode_complete_union(reader, 0)?, Span::zero()));
    }

    let mixin_count = reader.read_count("class-like mixins", MAXIMUM_MUTATIONS)?;
    for _ in 0..mixin_count {
        metadata.mixins.push(TypeMetadata::from_docblock(protocol::decode_complete_union(reader, 0)?, Span::zero()));
    }
    metadata.has_sealed_methods = read_optional_bool(reader)?;
    metadata.has_sealed_properties = read_optional_bool(reader)?;
    if reader.read_bool("permitted inheritors presence")? {
        metadata.permitted_inheritors = Some(read_name_set(reader, "permitted inheritors", true)?);
    }

    let method_count = reader.read_count("class methods", MAXIMUM_MUTATIONS)?;
    let mut methods = Vec::with_capacity(method_count);
    for _ in 0..method_count {
        let method = read_function_definition(reader, FunctionLikeKind::Method, Some(name))?;
        if !metadata.methods.insert(method.name) {
            return Err(protocol(format!(
                "class-like `{original_name}` contains duplicate method `{}`",
                method.original_name
            )));
        }
        methods.push(method);
    }

    let property_count = reader.read_count("class properties", MAXIMUM_MUTATIONS)?;
    for _ in 0..property_count {
        let property = read_property_definition(reader)?;
        if metadata.properties.insert(property.name.0, property).is_some() {
            return Err(protocol(format!("class-like `{original_name}` contains a duplicate property")));
        }
    }

    let magic_property_count = reader.read_count("class magic properties", MAXIMUM_MUTATIONS)?;
    for _ in 0..magic_property_count {
        let property = read_property_definition(reader)?;
        if metadata.magic_properties.insert(property.name.0, property).is_some() {
            return Err(protocol(format!("class-like `{original_name}` contains a duplicate magic property")));
        }
    }

    let constant_count = reader.read_count("class constants", MAXIMUM_MUTATIONS)?;
    for _ in 0..constant_count {
        let constant = read_class_constant_definition(reader)?;
        if metadata.constants.insert(constant.name, constant).is_some() {
            return Err(protocol(format!("class-like `{original_name}` contains a duplicate class constant")));
        }
    }

    let case_count = reader.read_count("enum cases", MAXIMUM_MUTATIONS)?;
    for _ in 0..case_count {
        let case = read_enum_case_definition(reader)?;
        if metadata.enum_cases.insert(case.name, case).is_some() {
            return Err(protocol(format!("class-like `{original_name}` contains a duplicate enum case")));
        }
    }

    Ok((metadata, methods))
}

fn read_function_definition(
    reader: &mut PayloadReader<'_>,
    kind: FunctionLikeKind,
    defining_class: Option<Word>,
) -> Result<FunctionLikeMetadata, ExternalAnalyzerError> {
    let original_name = read_name(reader, "function-like name")?;
    let name = ascii_lowercase_word(original_name.as_bytes());
    let mut metadata = FunctionLikeMetadata::new(kind, name, original_name, Span::zero(), read_flags(reader)?);
    let count = reader.read_count("function-like parameters", MAXIMUM_MUTATIONS)?;
    for _ in 0..count {
        metadata.parameters.push(read_parameter_definition(reader)?);
    }
    metadata.return_type_declaration_metadata = read_optional_type_metadata(reader, false)?;
    metadata.return_type_metadata = read_optional_type_metadata(reader, true)?;
    let thrown_count = reader.read_count("thrown types", MAXIMUM_MUTATIONS)?;
    for _ in 0..thrown_count {
        metadata
            .thrown_types
            .push(TypeMetadata::from_docblock(protocol::decode_complete_union(reader, 0)?, Span::zero()));
    }
    let defining_entity = defining_class
        .map_or(GenericParent::FunctionLike((name, empty_word())), |class| GenericParent::FunctionLike((class, name)));
    for (template_name, template, _, _) in read_templates(reader, defining_entity)? {
        metadata.template_types.insert(template_name, template);
    }
    metadata.has_docblock = metadata.return_type_metadata.is_some()
        || !metadata.thrown_types.is_empty()
        || !metadata.template_types.is_empty();

    if let Some(method) = metadata.method_metadata.as_mut() {
        method.visibility = read_visibility(reader)?;
        method.is_final = metadata.flags.is_final();
        method.is_abstract = metadata.flags.is_abstract();
        method.is_static = metadata.flags.is_static();
        method.is_constructor = name.as_bytes().eq_ignore_ascii_case(b"__construct");
    }
    Ok(metadata)
}

fn read_parameter_definition(
    reader: &mut PayloadReader<'_>,
) -> Result<FunctionLikeParameterMetadata, ExternalAnalyzerError> {
    let name = read_name(reader, "parameter name")?;
    if !name.as_bytes().starts_with(b"$") {
        return Err(protocol(format!("parameter `{name}` must begin with `$`")));
    }
    let mut metadata =
        FunctionLikeParameterMetadata::new(VariableIdentifier(name), Span::zero(), Span::zero(), read_flags(reader)?);
    metadata.type_declaration_metadata = read_optional_type_metadata(reader, false)?;
    metadata.type_metadata = read_optional_type_metadata(reader, true)?;
    metadata.out_type = read_optional_type_metadata(reader, true)?;
    metadata.closure_this_type = read_optional_type_metadata(reader, true)?;
    metadata.default_type = read_optional_type_metadata(reader, false)?;
    Ok(metadata)
}

fn read_property_definition(reader: &mut PayloadReader<'_>) -> Result<PropertyMetadata, ExternalAnalyzerError> {
    let name = read_name(reader, "property name")?;
    if !name.as_bytes().starts_with(b"$") {
        return Err(protocol(format!("property `{name}` must begin with `$`")));
    }
    let mut metadata = PropertyMetadata::new(VariableIdentifier(name), read_flags(reader)?);
    metadata.read_visibility = read_visibility(reader)?;
    metadata.write_visibility = read_visibility(reader)?;
    metadata.type_declaration_metadata = read_optional_type_metadata(reader, false)?;
    metadata.type_metadata = read_optional_type_metadata(reader, true)?;
    metadata.write_type_metadata = read_optional_type_metadata(reader, true)?;
    metadata.default_type_metadata = read_optional_type_metadata(reader, false)?;
    Ok(metadata)
}

fn read_class_constant_definition(
    reader: &mut PayloadReader<'_>,
) -> Result<ClassLikeConstantMetadata, ExternalAnalyzerError> {
    let name = read_name(reader, "class constant name")?;
    let mut metadata =
        ClassLikeConstantMetadata::new(name, Span::zero(), read_visibility(reader)?, read_flags(reader)?);
    metadata.type_declaration = read_optional_type_metadata(reader, false)?;
    metadata.type_metadata = read_optional_type_metadata(reader, true)?;
    metadata.inferred_type = read_optional_atomic(reader, "class constant inferred type")?;
    Ok(metadata)
}

fn read_enum_case_definition(reader: &mut PayloadReader<'_>) -> Result<EnumCaseMetadata, ExternalAnalyzerError> {
    let name = read_name(reader, "enum case name")?;
    let mut metadata = EnumCaseMetadata::new(name, Span::zero(), Span::zero(), read_flags(reader)?);
    metadata.value_type = read_optional_atomic(reader, "enum case value type")?;
    Ok(metadata)
}

fn read_constant_definition(reader: &mut PayloadReader<'_>) -> Result<ConstantMetadata, ExternalAnalyzerError> {
    let name = read_name(reader, "constant name")?;
    let mut metadata = ConstantMetadata::new(name, Span::zero(), read_flags(reader)?);
    metadata.type_metadata = read_optional_type_metadata(reader, true)?;
    metadata.inferred_type = read_optional_union(reader)?;
    Ok(metadata)
}

fn read_name(reader: &mut PayloadReader<'_>, description: &'static str) -> Result<Word, ExternalAnalyzerError> {
    let value = reader.read_bytes(description)?;
    if value.is_empty() {
        return Err(protocol(format!("{description} cannot be empty")));
    }
    Ok(word(value))
}

fn read_optional_name(
    reader: &mut PayloadReader<'_>,
    description: &'static str,
) -> Result<Option<Word>, ExternalAnalyzerError> {
    if reader.read_bool(description)? { read_name(reader, description).map(Some) } else { Ok(None) }
}

fn read_name_set(
    reader: &mut PayloadReader<'_>,
    description: &'static str,
    lowercase: bool,
) -> Result<mago_word::WordSet, ExternalAnalyzerError> {
    let count = reader.read_count(description, MAXIMUM_MUTATIONS)?;
    let mut names = mago_word::WordSet::default();
    for _ in 0..count {
        let name = read_name(reader, description)?;
        names.insert(if lowercase { ascii_lowercase_word(name.as_bytes()) } else { name });
    }
    Ok(names)
}

fn read_symbol_kind(reader: &mut PayloadReader<'_>) -> Result<SymbolKind, ExternalAnalyzerError> {
    match reader.read_u8("class-like kind")? {
        1 => Ok(SymbolKind::Class),
        2 => Ok(SymbolKind::Interface),
        3 => Ok(SymbolKind::Trait),
        4 => Ok(SymbolKind::Enum),
        unknown => Err(protocol(format!("unknown class-like kind {unknown}"))),
    }
}

fn read_visibility(reader: &mut PayloadReader<'_>) -> Result<Visibility, ExternalAnalyzerError> {
    match reader.read_u8("visibility")? {
        1 => Ok(Visibility::Public),
        2 => Ok(Visibility::Protected),
        3 => Ok(Visibility::Private),
        unknown => Err(protocol(format!("unknown visibility {unknown}"))),
    }
}

fn read_flags(reader: &mut PayloadReader<'_>) -> Result<MetadataFlags, ExternalAnalyzerError> {
    let mut flags = MetadataFlags::from_bits(reader.read_u64("metadata flags")?);
    flags.remove(MetadataFlags::POPULATED | MetadataFlags::BUILTIN | MetadataFlags::PATCH);
    flags.insert(MetadataFlags::USER_DEFINED);
    Ok(flags)
}

fn read_templates(
    reader: &mut PayloadReader<'_>,
    defining_entity: GenericParent,
) -> Result<Vec<(Word, GenericTemplate, Variance, bool)>, ExternalAnalyzerError> {
    let count = reader.read_count("generic templates", MAXIMUM_MUTATIONS)?;
    let mut templates = Vec::with_capacity(count);
    let mut names = HashSet::default();
    for _ in 0..count {
        let name = read_name(reader, "template name")?;
        if !names.insert(name) {
            return Err(protocol(format!("generic templates contain duplicate `{name}`")));
        }
        let constraint = protocol::decode_complete_union(reader, 0)?;
        let default = read_optional_union(reader)?;
        let variance = match reader.read_u8("template variance")? {
            1 => Variance::Invariant,
            2 => Variance::Covariant,
            3 => Variance::Contravariant,
            4 => Variance::Bivariant,
            unknown => return Err(protocol(format!("unknown template variance {unknown}"))),
        };
        let readonly = reader.read_bool("readonly template")?;
        templates.push((
            name,
            GenericTemplate::new(defining_entity, constraint).with_default(default),
            variance,
            readonly,
        ));
    }
    Ok(templates)
}

fn read_optional_bool(reader: &mut PayloadReader<'_>) -> Result<Option<bool>, ExternalAnalyzerError> {
    match reader.read_u8("optional boolean")? {
        0 => Ok(None),
        1 => Ok(Some(false)),
        2 => Ok(Some(true)),
        unknown => Err(protocol(format!("unknown optional boolean {unknown}"))),
    }
}

fn read_optional_type_metadata(
    reader: &mut PayloadReader<'_>,
    from_docblock: bool,
) -> Result<Option<TypeMetadata>, ExternalAnalyzerError> {
    if !reader.read_bool("optional type")? {
        return Ok(None);
    }
    let union = protocol::decode_complete_union(reader, 0)?;
    Ok(Some(if from_docblock {
        TypeMetadata::from_docblock(union, Span::zero())
    } else {
        TypeMetadata::new(union, Span::zero())
    }))
}

fn read_optional_union(reader: &mut PayloadReader<'_>) -> Result<Option<TUnion>, ExternalAnalyzerError> {
    if reader.read_bool("optional type")? { protocol::decode_complete_union(reader, 0).map(Some) } else { Ok(None) }
}

fn read_optional_atomic(
    reader: &mut PayloadReader<'_>,
    description: &'static str,
) -> Result<Option<TAtomic>, ExternalAnalyzerError> {
    let Some(union) = read_optional_union(reader)? else {
        return Ok(None);
    };
    let mut atomics = union.types.into_owned();
    if atomics.len() != 1 {
        return Err(protocol(format!("{description} must contain exactly one atomic type")));
    }
    Ok(atomics.pop())
}

fn write_class_like_definition(
    writer: &mut PayloadWriter,
    metadata: &ClassLikeMetadata,
    codebase: &CodebaseMetadata,
) -> Result<(), ExternalAnalyzerError> {
    writer.write_bytes(metadata.original_name.as_bytes())?;
    writer.write_u8(match metadata.kind {
        SymbolKind::Class => 1,
        SymbolKind::Interface => 2,
        SymbolKind::Trait => 3,
        SymbolKind::Enum => 4,
    });
    writer.write_u64(metadata.flags.bits());
    write_optional_word(writer, metadata.direct_parent_class)?;
    write_words(writer, metadata.direct_parent_interfaces.iter().copied())?;
    write_words(writer, metadata.require_extends.iter().copied())?;
    write_words(writer, metadata.require_implements.iter().copied())?;
    write_words(writer, metadata.used_traits.iter().copied())?;
    write_optional_atomic(writer, metadata.enum_type.as_ref())?;
    write_templates(
        writer,
        &metadata.template_types,
        Some(&metadata.template_variance),
        Some(&metadata.template_readonly),
    )?;
    writer.write_u32(metadata.template_extended_offsets.len() as u32);
    for (parent, types) in &metadata.template_extended_offsets {
        writer.write_bytes(parent.as_bytes())?;
        writer.write_u32(types.len() as u32);
        for union in types {
            write_union(writer, union)?;
        }
    }
    writer.write_u32(metadata.type_aliases.len() as u32);
    for (name, alias) in &metadata.type_aliases {
        writer.write_bytes(name.as_bytes())?;
        write_union(writer, &alias.type_union)?;
    }
    writer.write_u32(metadata.mixins.len() as u32);
    for mixin in &metadata.mixins {
        write_union(writer, &mixin.type_union)?;
    }
    write_optional_bool(writer, metadata.has_sealed_methods);
    write_optional_bool(writer, metadata.has_sealed_properties);
    writer.write_bool(metadata.permitted_inheritors.is_some());
    if let Some(permitted) = &metadata.permitted_inheritors {
        write_words(writer, permitted.iter().copied())?;
    }
    writer.write_u32(metadata.methods.len() as u32);
    for method in &metadata.methods {
        let function = codebase.function_likes.get(&(metadata.name, *method)).ok_or_else(|| {
            protocol(format!("class-like `{}` is missing method metadata for `{method}`", metadata.original_name))
        })?;
        write_function_definition(writer, function)?;
    }
    writer.write_u32(metadata.properties.len() as u32);
    for property in metadata.properties.values() {
        write_property_definition(writer, property)?;
    }
    writer.write_u32(metadata.magic_properties.len() as u32);
    for property in metadata.magic_properties.values() {
        write_property_definition(writer, property)?;
    }
    writer.write_u32(metadata.constants.len() as u32);
    for constant in metadata.constants.values() {
        write_class_constant_definition(writer, constant)?;
    }
    writer.write_u32(metadata.enum_cases.len() as u32);
    for case in metadata.enum_cases.values() {
        write_enum_case_definition(writer, case)?;
    }
    Ok(())
}

fn write_function_definition(
    writer: &mut PayloadWriter,
    metadata: &FunctionLikeMetadata,
) -> Result<(), ExternalAnalyzerError> {
    writer.write_bytes(metadata.original_name.as_bytes())?;
    writer.write_u64(metadata.flags.bits());
    writer.write_u32(metadata.parameters.len() as u32);
    for parameter in &metadata.parameters {
        writer.write_bytes(parameter.name.0.as_bytes())?;
        writer.write_u64(parameter.flags.bits());
        write_optional_type_metadata(writer, parameter.type_declaration_metadata.as_ref())?;
        write_optional_type_metadata(writer, parameter.type_metadata.as_ref())?;
        write_optional_type_metadata(writer, parameter.out_type.as_ref())?;
        write_optional_type_metadata(writer, parameter.closure_this_type.as_ref())?;
        write_optional_type_metadata(writer, parameter.default_type.as_ref())?;
    }
    write_optional_type_metadata(writer, metadata.return_type_declaration_metadata.as_ref())?;
    write_optional_type_metadata(writer, metadata.return_type_metadata.as_ref())?;
    writer.write_u32(metadata.thrown_types.len() as u32);
    for thrown in &metadata.thrown_types {
        write_union(writer, &thrown.type_union)?;
    }
    write_templates(writer, &metadata.template_types, None, None)?;
    if metadata.kind == FunctionLikeKind::Method {
        write_visibility(
            writer,
            metadata.method_metadata.as_ref().map_or(Visibility::Public, |method| method.visibility),
        );
    }
    Ok(())
}

fn write_property_definition(
    writer: &mut PayloadWriter,
    metadata: &PropertyMetadata,
) -> Result<(), ExternalAnalyzerError> {
    writer.write_bytes(metadata.name.0.as_bytes())?;
    writer.write_u64(metadata.flags.bits());
    write_visibility(writer, metadata.read_visibility);
    write_visibility(writer, metadata.write_visibility);
    write_optional_type_metadata(writer, metadata.type_declaration_metadata.as_ref())?;
    write_optional_type_metadata(writer, metadata.type_metadata.as_ref())?;
    write_optional_type_metadata(writer, metadata.write_type_metadata.as_ref())?;
    write_optional_type_metadata(writer, metadata.default_type_metadata.as_ref())?;
    Ok(())
}

fn write_class_constant_definition(
    writer: &mut PayloadWriter,
    metadata: &ClassLikeConstantMetadata,
) -> Result<(), ExternalAnalyzerError> {
    writer.write_bytes(metadata.name.as_bytes())?;
    write_visibility(writer, metadata.visibility);
    writer.write_u64(metadata.flags.bits());
    write_optional_type_metadata(writer, metadata.type_declaration.as_ref())?;
    write_optional_type_metadata(writer, metadata.type_metadata.as_ref())?;
    write_optional_atomic(writer, metadata.inferred_type.as_ref())?;
    Ok(())
}

fn write_enum_case_definition(
    writer: &mut PayloadWriter,
    metadata: &EnumCaseMetadata,
) -> Result<(), ExternalAnalyzerError> {
    writer.write_bytes(metadata.name.as_bytes())?;
    writer.write_u64(metadata.flags.bits());
    write_optional_atomic(writer, metadata.value_type.as_ref())
}

fn write_constant_definition(
    writer: &mut PayloadWriter,
    metadata: &ConstantMetadata,
) -> Result<(), ExternalAnalyzerError> {
    writer.write_bytes(metadata.name.as_bytes())?;
    writer.write_u64(metadata.flags.bits());
    write_optional_type_metadata(writer, metadata.type_metadata.as_ref())?;
    write_optional_union(writer, metadata.inferred_type.as_ref())
}

fn write_words(writer: &mut PayloadWriter, words: impl IntoIterator<Item = Word>) -> Result<(), ExternalAnalyzerError> {
    let words = words.into_iter().collect::<Vec<_>>();
    writer.write_u32(words.len() as u32);
    for value in words {
        writer.write_bytes(value.as_bytes())?;
    }
    Ok(())
}

fn write_optional_word(writer: &mut PayloadWriter, value: Option<Word>) -> Result<(), ExternalAnalyzerError> {
    writer.write_bool(value.is_some());
    if let Some(value) = value {
        writer.write_bytes(value.as_bytes())?;
    }
    Ok(())
}

fn write_visibility(writer: &mut PayloadWriter, visibility: Visibility) {
    writer.write_u8(match visibility {
        Visibility::Public => 1,
        Visibility::Protected => 2,
        Visibility::Private => 3,
    });
}

fn write_templates(
    writer: &mut PayloadWriter,
    templates: &mago_codex::metadata::class_like::TemplateTypes,
    variances: Option<&[Variance]>,
    readonly: Option<&mago_word::WordSet>,
) -> Result<(), ExternalAnalyzerError> {
    writer.write_u32(templates.len() as u32);
    for (index, (name, template)) in templates.iter().enumerate() {
        writer.write_bytes(name.as_bytes())?;
        write_union(writer, &template.constraint)?;
        write_optional_union(writer, template.default.as_ref())?;
        writer.write_u8(match variances.and_then(|values| values.get(index)).copied().unwrap_or(Variance::Invariant) {
            Variance::Invariant => 1,
            Variance::Covariant => 2,
            Variance::Contravariant => 3,
            Variance::Bivariant => 4,
        });
        writer.write_bool(readonly.is_some_and(|names| names.contains(name)));
    }
    Ok(())
}

fn write_optional_bool(writer: &mut PayloadWriter, value: Option<bool>) {
    writer.write_u8(match value {
        None => 0,
        Some(false) => 1,
        Some(true) => 2,
    });
}

fn write_optional_type_metadata(
    writer: &mut PayloadWriter,
    metadata: Option<&TypeMetadata>,
) -> Result<(), ExternalAnalyzerError> {
    write_optional_union(writer, metadata.map(|metadata| &metadata.type_union))
}

fn write_optional_atomic(writer: &mut PayloadWriter, atomic: Option<&TAtomic>) -> Result<(), ExternalAnalyzerError> {
    writer.write_bool(atomic.is_some());
    if let Some(atomic) = atomic {
        write_union(writer, &TUnion::from_atomic(atomic.clone()))?;
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

fn write_union(writer: &mut PayloadWriter, union: &TUnion) -> Result<(), ExternalAnalyzerError> {
    let mut references = Vec::new();
    protocol::encode_union_snapshot(writer, union, &mut references, 0)
}
