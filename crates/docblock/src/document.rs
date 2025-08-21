use bumpalo::collections::Vec;
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Document<'arena> {
    pub span: Span,
    pub elements: Vec<'arena, Element<'arena>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub enum Element<'arena> {
    Text(Text<'arena>),
    Code(Code<'arena>),
    Tag(Tag<'arena>),
    Line(Span),
    Annotation(Annotation<'arena>),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Text<'arena> {
    pub span: Span,
    pub segments: Vec<'arena, TextSegment<'arena>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Code<'arena> {
    pub span: Span,
    pub directives: Vec<'arena, &'arena str>,
    pub content: &'arena str,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub enum TextSegment<'arena> {
    Paragraph { span: Span, content: &'arena str },
    InlineCode(Code<'arena>),
    InlineTag(Tag<'arena>),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Annotation<'arena> {
    pub span: Span,
    pub name: &'arena str,
    pub arguments: Option<&'arena str>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Tag<'arena> {
    pub span: Span,
    pub name: &'arena str,
    pub kind: TagKind,
    pub description: &'arena str,
    pub description_span: Span,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[non_exhaustive]
pub enum TagKind {
    Abstract,
    Access,
    Author,
    Category,
    Copyright,
    Deprecated,
    Example,
    Final,
    FileSource,
    Global,
    Ignore,
    Internal,
    License,
    Link,
    Method,
    Mixin,
    Name,
    Package,
    Param,
    Property,
    PropertyRead,
    PropertyWrite,
    SealProperties,
    NoSealProperties,
    SealMethods,
    NoSealMethods,
    ReadOnly,
    NoNamedArguments,
    Api,
    PsalmApi,
    Inheritors,
    PsalmInheritors,
    Return,
    See,
    Since,
    Static,
    StaticVar,
    SubPackage,
    Todo,
    Tutorial,
    Uses,
    Var,
    Throws,
    Version,
    ParamLaterInvokedCallable,
    ParamImmediatelyInvokedCallable,
    ParamClosureThis,
    TemplateExtends,
    Extends,
    TemplateImplements,
    Implements,
    TemplateUse,
    Use,
    NotDeprecated,
    PhpstanImpure,
    PhpstanPure,
    Pure,
    Immutable,
    RequireExtends,
    RequireImplements,
    InheritDoc,
    ParamOut,
    Assert,
    AssertIfTrue,
    AssertIfFalse,
    ConsistentConstructor,
    PsalmConsistentConstructor,
    PsalmConsistentTemplates,
    PsalmParamOut,
    PsalmVar,
    PsalmParam,
    PsalmReturn,
    PsalmProperty,
    PsalmPropertyRead,
    PsalmPropertyWrite,
    PsalmMethod,
    PsalmIgnoreVar,
    PsalmSuppress,
    PsalmAssert,
    PsalmAssertIfTrue,
    PsalmAssertIfFalse,
    PsalmIfThisIs,
    PsalmThisOut,
    IgnoreNullableReturn,
    IgnoreFalsableReturn,
    PsalmIgnoreNullableReturn,
    PsalmIgnoreFalsableReturn,
    PsalmSealProperties,
    PsalmNoSealProperties,
    PsalmSealMethods,
    PsalmNoSealMethods,
    PsalmInternal,
    PsalmReadOnly,
    PsalmMutationFree,
    PsalmExternalMutationFree,
    MutationFree,
    ExternalMutationFree,
    PsalmImmutable,
    PsalmPure,
    PsalmAllowPrivateMutation,
    PsalmReadOnlyAllowPrivateMutation,
    PsalmTrace,
    PsalmCheckType,
    PsalmCheckTypeExact,
    PsalmTaintSource,
    PsalmTaintSink,
    PsalmTaintEscape,
    PsalmTaintUnescape,
    PsalmTaintSpecialize,
    PsalmFlow,
    Type,
    PsalmType,
    ImportType,
    PsalmImportType,
    PsalmRequireExtends,
    PsalmRequireImplements,
    PsalmIgnoreVariableProperty,
    PsalmIgnoreVariableMethod,
    PsalmYield,
    PhpstanAssert,
    PhpstanAssertIfTrue,
    PhpstanAssertIfFalse,
    PhpstanSelfOut,
    PhpstanThisOut,
    PhpstanRequireExtends,
    PhpstanRequireImplements,
    PhpstanParam,
    PhpstanReturn,
    PhpstanVar,
    PhpstanReadOnly,
    PhpstanImmutable,
    Template,
    TemplateInvariant,
    TemplateCovariant,
    TemplateContravariant,
    PsalmTemplate,
    PsalmTemplateInvariant,
    PsalmTemplateCovariant,
    PsalmTemplateContravariant,
    PhpstanTemplate,
    PhpstanTemplateInvariant,
    PhpstanTemplateCovariant,
    PhpstanTemplateContravariant,
    EnumInterface,
    MagoUnchecked,
    Unchecked,
    ThisOut,
    SelfOut,
    Where,
    MustUse,
    Other,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[repr(u8)]
pub enum TagVendor {
    Mago,
    Phpstan,
    Psalm,
}

impl<'arena> Document<'arena> {
    pub fn get_tags(&self) -> impl Iterator<Item = &Tag<'arena>> {
        self.elements.iter().filter_map(|element| if let Element::Tag(tag) = element { Some(tag) } else { None })
    }

    pub fn get_tags_by_kind(&self, kind: TagKind) -> impl Iterator<Item = &Tag<'arena>> {
        self.get_tags().filter(move |tag| tag.kind == kind)
    }
}

impl HasSpan for Document<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl TagKind {
    /// Returns the vendor of the tag, if it has one.
    ///
    /// If the tag does not have a vendor, `None` is returned.
    pub fn get_vendor(&self) -> Option<TagVendor> {
        match self {
            Self::PsalmConsistentConstructor
            | Self::PsalmConsistentTemplates
            | Self::PsalmParamOut
            | Self::PsalmVar
            | Self::PsalmParam
            | Self::PsalmReturn
            | Self::PsalmProperty
            | Self::PsalmPropertyRead
            | Self::PsalmPropertyWrite
            | Self::PsalmMethod
            | Self::PsalmIgnoreVar
            | Self::PsalmSuppress
            | Self::PsalmAssert
            | Self::PsalmAssertIfTrue
            | Self::PsalmAssertIfFalse
            | Self::PsalmIfThisIs
            | Self::PsalmThisOut
            | Self::PsalmIgnoreNullableReturn
            | Self::PsalmIgnoreFalsableReturn
            | Self::PsalmSealProperties
            | Self::PsalmNoSealProperties
            | Self::PsalmSealMethods
            | Self::PsalmNoSealMethods
            | Self::PsalmInternal
            | Self::PsalmReadOnly
            | Self::PsalmMutationFree
            | Self::PsalmExternalMutationFree
            | Self::PsalmImmutable
            | Self::PsalmPure
            | Self::PsalmAllowPrivateMutation
            | Self::PsalmReadOnlyAllowPrivateMutation
            | Self::PsalmTrace
            | Self::PsalmCheckType
            | Self::PsalmCheckTypeExact
            | Self::PsalmTaintSource
            | Self::PsalmTaintSink
            | Self::PsalmTaintEscape
            | Self::PsalmTaintUnescape
            | Self::PsalmTaintSpecialize
            | Self::PsalmFlow
            | Self::PsalmType
            | Self::PsalmRequireExtends
            | Self::PsalmRequireImplements
            | Self::PsalmIgnoreVariableProperty
            | Self::PsalmIgnoreVariableMethod
            | Self::PsalmYield
            | Self::PsalmTemplate
            | Self::PsalmTemplateInvariant
            | Self::PsalmTemplateCovariant
            | Self::PsalmTemplateContravariant
            | Self::PsalmInheritors
            | Self::PsalmImportType => Some(TagVendor::Psalm),
            Self::PhpstanAssert
            | Self::PhpstanAssertIfTrue
            | Self::PhpstanAssertIfFalse
            | Self::PhpstanSelfOut
            | Self::PhpstanThisOut
            | Self::PhpstanRequireExtends
            | Self::PhpstanRequireImplements
            | Self::PhpstanTemplate
            | Self::PhpstanTemplateInvariant
            | Self::PhpstanTemplateCovariant
            | Self::PhpstanTemplateContravariant
            | Self::PhpstanParam
            | Self::PhpstanReturn
            | Self::PhpstanVar
            | Self::PhpstanReadOnly
            | Self::PhpstanImmutable => Some(TagVendor::Phpstan),
            Self::MagoUnchecked => Some(TagVendor::Mago),
            _ => None,
        }
    }

    /// Returns the non-vendored variant of the tag, if it exists.
    ///
    /// Note that not all vendored tags have a non-vendored variant.
    ///
    /// If the tag is not vendored, or if it does not have a non-vendored variant,
    ///  `None` is returned.
    pub fn get_non_vendored_variant(&self) -> Option<TagKind> {
        match self {
            Self::PsalmConsistentConstructor => Some(Self::ConsistentConstructor),
            Self::PsalmParamOut => Some(Self::ParamOut),
            Self::PsalmVar => Some(Self::Var),
            Self::PsalmParam => Some(Self::Param),
            Self::PsalmReturn => Some(Self::Return),
            Self::PsalmProperty => Some(Self::Property),
            Self::PsalmPropertyRead => Some(Self::PropertyRead),
            Self::PsalmPropertyWrite => Some(Self::PropertyWrite),
            Self::PsalmMethod => Some(Self::Method),
            Self::PsalmSealProperties => Some(Self::SealProperties),
            Self::PsalmNoSealProperties => Some(Self::NoSealProperties),
            Self::PsalmSealMethods => Some(Self::SealMethods),
            Self::PsalmNoSealMethods => Some(Self::NoSealMethods),
            Self::PsalmInternal => Some(Self::Internal),
            Self::PsalmReadOnly => Some(Self::ReadOnly),
            Self::PsalmImmutable => Some(Self::Immutable),
            Self::PsalmPure => Some(Self::Pure),
            Self::PhpstanParam => Some(Self::Param),
            Self::PhpstanReturn => Some(Self::Return),
            Self::PhpstanVar => Some(Self::Var),
            Self::PhpstanReadOnly => Some(Self::ReadOnly),
            Self::PhpstanImmutable => Some(Self::Immutable),
            Self::PhpstanAssert | Self::PsalmAssert => Some(Self::Assert),
            Self::PhpstanAssertIfTrue | Self::PsalmAssertIfTrue => Some(Self::AssertIfTrue),
            Self::PhpstanAssertIfFalse | Self::PsalmAssertIfFalse => Some(Self::AssertIfFalse),
            Self::PhpstanTemplate | Self::PsalmTemplate => Some(Self::Template),
            Self::PhpstanTemplateInvariant | Self::PsalmTemplateInvariant => Some(Self::TemplateInvariant),
            Self::PhpstanTemplateCovariant | Self::PsalmTemplateCovariant => Some(Self::TemplateCovariant),
            Self::PhpstanTemplateContravariant | Self::PsalmTemplateContravariant => Some(Self::TemplateContravariant),
            Self::PsalmMutationFree => Some(Self::MutationFree),
            Self::PsalmExternalMutationFree => Some(Self::ExternalMutationFree),
            Self::PsalmIgnoreFalsableReturn => Some(Self::IgnoreFalsableReturn),
            Self::PsalmIgnoreNullableReturn => Some(Self::IgnoreNullableReturn),
            Self::PsalmInheritors => Some(Self::Inheritors),
            Self::MagoUnchecked => Some(Self::Unchecked),
            Self::PsalmType => Some(Self::Type),
            Self::PsalmImportType => Some(Self::ImportType),
            Self::PhpstanRequireExtends | Self::PsalmRequireExtends => Some(Self::RequireExtends),
            Self::PhpstanRequireImplements | Self::PsalmRequireImplements => Some(Self::RequireImplements),
            Self::PsalmThisOut | Self::PhpstanThisOut => Some(Self::ThisOut),
            Self::PhpstanSelfOut => Some(Self::SelfOut),
            _ => None,
        }
    }

    pub fn is_repeatable(&self) -> bool {
        matches!(
            self,
            Self::Author
                | Self::Deprecated
                | Self::Example
                | Self::Ignore
                | Self::Link
                | Self::Method
                | Self::Mixin
                | Self::Package
                | Self::Param
                | Self::Property
                | Self::PropertyRead
                | Self::PropertyWrite
                | Self::Return
                | Self::See
                | Self::Since
                | Self::Throws
                | Self::Uses
                | Self::Var
                | Self::Template
                | Self::TemplateInvariant
                | Self::TemplateCovariant
                | Self::TemplateContravariant
                | Self::PsalmTemplate
                | Self::PsalmTemplateInvariant
                | Self::PsalmTemplateCovariant
                | Self::PsalmTemplateContravariant
                | Self::PhpstanTemplate
                | Self::PhpstanTemplateInvariant
                | Self::PhpstanTemplateCovariant
                | Self::PhpstanTemplateContravariant
                | Self::PhpstanParam
                | Self::PhpstanVar
                | Self::PsalmVar
                | Self::PsalmParam
                | Self::Extends
                | Self::TemplateExtends
                | Self::Implements
                | Self::TemplateImplements
                | Self::Use
                | Self::TemplateUse
                | Self::PsalmType
                | Self::Type
                | Self::PsalmImportType
                | Self::RequireImplements
                | Self::PsalmRequireImplements
                | Self::PhpstanRequireImplements
                | Self::RequireExtends
                | Self::PsalmRequireExtends
                | Self::PhpstanRequireExtends
                | Self::Where
        )
    }
}

impl<T> From<T> for TagKind
where
    T: AsRef<str>,
{
    fn from(value: T) -> Self {
        match value.as_ref().to_ascii_lowercase().as_str() {
            "abstract" => TagKind::Abstract,
            "access" => TagKind::Access,
            "author" => TagKind::Author,
            "category" => TagKind::Category,
            "copyright" => TagKind::Copyright,
            "deprecated" => TagKind::Deprecated,
            "example" => TagKind::Example,
            "final" => TagKind::Final,
            "filesource" => TagKind::FileSource,
            "global" => TagKind::Global,
            "ignore" => TagKind::Ignore,
            "internal" => TagKind::Internal,
            "license" => TagKind::License,
            "link" => TagKind::Link,
            "method" => TagKind::Method,
            "mixin" => TagKind::Mixin,
            "name" => TagKind::Name,
            "package" => TagKind::Package,
            "param" => TagKind::Param,
            "property" => TagKind::Property,
            "property-read" => TagKind::PropertyRead,
            "propertyread" => TagKind::PropertyRead,
            "property-write" => TagKind::PropertyWrite,
            "propertywrite" => TagKind::PropertyWrite,
            "sealproperties" => TagKind::SealProperties,
            "seal-properties" => TagKind::SealProperties,
            "nosealproperties" => TagKind::NoSealProperties,
            "no-seal-properties" => TagKind::NoSealProperties,
            "sealmethods" => TagKind::SealMethods,
            "seal-methods" => TagKind::SealMethods,
            "nosealmethods" => TagKind::NoSealMethods,
            "no-seal-methods" => TagKind::NoSealMethods,
            "readonly" => TagKind::ReadOnly,
            "nonamedarguments" => TagKind::NoNamedArguments,
            "no-named-arguments" => TagKind::NoNamedArguments,
            "api" => TagKind::Api,
            "psalm-api" => TagKind::PsalmApi,
            "psalm-inheritors" | "psalminheritors" => TagKind::PsalmInheritors,
            "inheritors" => TagKind::Inheritors,
            "return" => TagKind::Return,
            "see" => TagKind::See,
            "since" => TagKind::Since,
            "static" => TagKind::Static,
            "staticvar" => TagKind::StaticVar,
            "static-var" => TagKind::StaticVar,
            "subpackage" => TagKind::SubPackage,
            "sub-package" => TagKind::SubPackage,
            "todo" => TagKind::Todo,
            "tutorial" => TagKind::Tutorial,
            "uses" => TagKind::Uses,
            "var" => TagKind::Var,
            "throws" => TagKind::Throws,
            "version" => TagKind::Version,
            "assert" => TagKind::Assert,
            "assert-if-true" | "assertiftrue" => TagKind::AssertIfTrue,
            "assert-if-false" | "assertiffalse" => TagKind::AssertIfFalse,
            "param-later-invoked-callable" => TagKind::ParamLaterInvokedCallable,
            "paramlaterinvokedcallable" => TagKind::ParamLaterInvokedCallable,
            "param-immediately-invoked-callable" => TagKind::ParamImmediatelyInvokedCallable,
            "paramimmediatelyinvokedcallable" => TagKind::ParamImmediatelyInvokedCallable,
            "param-closure-this" => TagKind::ParamClosureThis,
            "paramclosurethis" => TagKind::ParamClosureThis,
            "extends" => TagKind::Extends,
            "template-extends" | "templateextends" => TagKind::TemplateExtends,
            "implements" => TagKind::Implements,
            "template-implements" | "templateimplements" => TagKind::TemplateImplements,
            "use" => TagKind::Use,
            "template-use" | "templateuse" => TagKind::TemplateUse,
            "not-deprecated" | "notdeprecated" => TagKind::NotDeprecated,
            "phpstan-impure" | "phpstanimpure" => TagKind::PhpstanImpure,
            "phpstan-pure" | "phpstanpure" => TagKind::PhpstanPure,
            "pure" => TagKind::Pure,
            "immutable" => TagKind::Immutable,
            "inheritdoc" => TagKind::InheritDoc,
            "inherit-doc" => TagKind::InheritDoc,
            "param-out" => TagKind::ParamOut,
            "psalm-param-out" => TagKind::PsalmParamOut,
            "consistentconstructor" | "consistent-constructor" => TagKind::ConsistentConstructor,
            "psalmconsistentconstructor" | "psalm-consistent-constructor" => TagKind::PsalmConsistentConstructor,
            "psalmconsistenttemplates" | "psalm-consistent-templates" => TagKind::PsalmConsistentTemplates,
            "psalm-var" => TagKind::PsalmVar,
            "psalm-param" => TagKind::PsalmParam,
            "psalm-return" => TagKind::PsalmReturn,
            "psalm-property" => TagKind::PsalmProperty,
            "psalm-property-read" => TagKind::PsalmPropertyRead,
            "psalm-propertyread" => TagKind::PsalmPropertyRead,
            "psalm-property-write" => TagKind::PsalmPropertyWrite,
            "psalm-propertywrite" => TagKind::PsalmPropertyWrite,
            "psalm-method" => TagKind::PsalmMethod,
            "psalm-ignore-var" => TagKind::PsalmIgnoreVar,
            "psalmignorevar" => TagKind::PsalmIgnoreVar,
            "psalm-suppress" => TagKind::PsalmSuppress,
            "psalm-assert" => TagKind::PsalmAssert,
            "psalm-assert-if-true" | "psalmassertiftrue" => TagKind::PsalmAssertIfTrue,
            "psalm-assert-if-false" | "psalmassertiffalse" => TagKind::PsalmAssertIfFalse,
            "psalm-if-this-is" | "psalmifthisis" => TagKind::PsalmIfThisIs,
            "psalm-this-out" | "psalmthisout" => TagKind::PsalmThisOut,
            "ignore-nullable-return" | "ignorenullablereturn" => TagKind::IgnoreNullableReturn,
            "ignore-falsable-return" | "ignorefalsablereturn" => TagKind::IgnoreFalsableReturn,
            "psalm-ignore-nullable-return" | "psalmignorenullablereturn" => TagKind::PsalmIgnoreNullableReturn,
            "psalm-ignore-falsable-return" | "psalmignorefalsablereturn" => TagKind::PsalmIgnoreFalsableReturn,
            "psalm-seal-properties" => TagKind::PsalmSealProperties,
            "psalmsealproperties" => TagKind::PsalmSealProperties,
            "psalm-no-seal-properties" => TagKind::PsalmNoSealProperties,
            "psalmnosealproperties" => TagKind::PsalmNoSealProperties,
            "psalm-seal-methods" => TagKind::PsalmSealMethods,
            "psalmsealmethods" => TagKind::PsalmSealMethods,
            "psalm-no-seal-methods" => TagKind::PsalmNoSealMethods,
            "psalmnosealmethods" => TagKind::PsalmNoSealMethods,
            "psalm-internal" => TagKind::PsalmInternal,
            "psalm-readonly" => TagKind::PsalmReadOnly,
            "psalm-mutation-free" | "psalmmutationfree" => TagKind::PsalmMutationFree,
            "psalm-external-mutation-free" | "psalmexternalmutationfree" => TagKind::PsalmExternalMutationFree,
            "mutation-free" | "mutationfree" => TagKind::MutationFree,
            "external-mutation-free" | "externalmutationfree" => TagKind::ExternalMutationFree,
            "psalm-immutable" => TagKind::PsalmImmutable,
            "psalm-pure" => TagKind::PsalmPure,
            "psalm-allow-private-mutation" => TagKind::PsalmAllowPrivateMutation,
            "psalmallowprivatemutation" => TagKind::PsalmAllowPrivateMutation,
            "psalm-readonly-allow-private-mutation" => TagKind::PsalmReadOnlyAllowPrivateMutation,
            "psalmreadonlyallowprivatemutation" => TagKind::PsalmReadOnlyAllowPrivateMutation,
            "psalm-trace" => TagKind::PsalmTrace,
            "psalm-check-type" => TagKind::PsalmCheckType,
            "psalmchecktype" => TagKind::PsalmCheckType,
            "psalm-check-type-exact" => TagKind::PsalmCheckTypeExact,
            "psalmchecktypeexact" => TagKind::PsalmCheckTypeExact,
            "psalm-taint-source" => TagKind::PsalmTaintSource,
            "psalmtaintsource" => TagKind::PsalmTaintSource,
            "psalm-taint-sink" => TagKind::PsalmTaintSink,
            "psalmtaintsink" => TagKind::PsalmTaintSink,
            "psalm-taint-escape" => TagKind::PsalmTaintEscape,
            "psalmtaintescape" => TagKind::PsalmTaintEscape,
            "psalm-taint-unescape" => TagKind::PsalmTaintUnescape,
            "psalmtaintunescape" => TagKind::PsalmTaintUnescape,
            "psalm-taint-specialize" => TagKind::PsalmTaintSpecialize,
            "psalmtaintspecialize" => TagKind::PsalmTaintSpecialize,
            "psalm-flow" => TagKind::PsalmFlow,
            "psalmflow" => TagKind::PsalmFlow,
            "psalm-type" | "psalmtype" => TagKind::PsalmType,
            "psalm-import-type" | "psalmimporttype" => TagKind::PsalmImportType,
            "psalm-require-extends" => TagKind::PsalmRequireExtends,
            "psalmrequireextends" => TagKind::PsalmRequireExtends,
            "psalm-require-implements" => TagKind::PsalmRequireImplements,
            "psalmrequireimplements" => TagKind::PsalmRequireImplements,
            "psalm-ignore-variable-property" => TagKind::PsalmIgnoreVariableProperty,
            "psalmignorevariableproperty" => TagKind::PsalmIgnoreVariableProperty,
            "psalm-ignore-variable-method" => TagKind::PsalmIgnoreVariableMethod,
            "psalmignorevariablemethod" => TagKind::PsalmIgnoreVariableMethod,
            "psalm-yield" => TagKind::PsalmYield,
            "phpstan-assert" => TagKind::PhpstanAssert,
            "phpstan-assert-if-true" => TagKind::PhpstanAssertIfTrue,
            "phpstan-assert-if-false" => TagKind::PhpstanAssertIfFalse,
            "phpstan-self-out" | "phpstanselfout" => TagKind::PhpstanSelfOut,
            "phpstan-this-out" | "phpstanthisout" => TagKind::PhpstanThisOut,
            "phpstan-require-extends" | "phpstanrequireextends" => TagKind::PhpstanRequireExtends,
            "phpstan-require-implements" | "phpstanrequireimplements" => TagKind::PhpstanRequireImplements,
            "template" => TagKind::Template,
            "template-invariant" | "templateinvariant" => TagKind::TemplateInvariant,
            "template-covariant" | "templatecovariant" => TagKind::TemplateCovariant,
            "template-contravariant" | "templatecontravariant" => TagKind::TemplateContravariant,
            "psalm-template" | "psalmtemplate" => TagKind::PsalmTemplate,
            "psalm-template-invariant" | "psalmtemplateinvariant" => TagKind::PsalmTemplateInvariant,
            "psalm-template-covariant" | "psalmtemplatecovariant" => TagKind::PsalmTemplateCovariant,
            "psalm-template-contravariant" | "psalmtemplatecontravariant" => TagKind::PsalmTemplateContravariant,
            "phpstan-template" | "phpstantemplate" => TagKind::PhpstanTemplate,
            "phpstan-template-invariant" | "phpstantemplateinvariant" => TagKind::PhpstanTemplateInvariant,
            "phpstan-template-covariant" | "phpstantemplatecovariant" => TagKind::PhpstanTemplateCovariant,
            "phpstan-template-contravariant" | "phpstantemplatecontravariant" => TagKind::PhpstanTemplateContravariant,
            "phpstan-param" => TagKind::PhpstanParam,
            "phpstan-return" => TagKind::PhpstanReturn,
            "phpstan-var" => TagKind::PhpstanVar,
            "phpstan-readonly" => TagKind::PhpstanReadOnly,
            "phpstan-immutable" => TagKind::PhpstanImmutable,
            "enuminterface" | "enum-interface" => TagKind::EnumInterface,
            "mago-unchecked" | "magounchecked" => TagKind::MagoUnchecked,
            "unchecked" => TagKind::Unchecked,
            "type" => TagKind::Type,
            "import-type" | "importtype" => TagKind::ImportType,
            "require-implements" | "requireimplements" => TagKind::RequireImplements,
            "require-extends" | "requireextends" => TagKind::RequireExtends,
            "self-out" | "selfout" => TagKind::SelfOut,
            "this-out" | "thisout" => TagKind::ThisOut,
            "where" => TagKind::Where,
            "must-use" | "mustuse" => TagKind::MustUse,
            _ => TagKind::Other,
        }
    }
}

impl TagVendor {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Mago => "mago",
            Self::Phpstan => "phpstan",
            Self::Psalm => "psalm",
        }
    }
}
