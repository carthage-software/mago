use mago_span::HasSpan;
use mago_span::Span;

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Document<'arena> {
    pub span: Span,
    pub elements: &'arena [Element<'arena>],
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Element<'arena> {
    Text(Text<'arena>),
    Code(Code<'arena>),
    Tag(Tag<'arena>),
    Line(Span),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Text<'arena> {
    pub span: Span,
    pub segments: &'arena [TextSegment<'arena>],
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Code<'arena> {
    pub span: Span,
    pub directives: &'arena [&'arena [u8]],
    pub content: &'arena [u8],
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum TextSegment<'arena> {
    Paragraph { span: Span, content: &'arena [u8] },
    InlineCode(Code<'arena>),
    InlineTag(Tag<'arena>),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Tag<'arena> {
    pub span: Span,
    pub name: &'arena [u8],
    pub kind: TagKind,
    pub metadata: Option<&'arena [u8]>,
    pub description: &'arena [u8],
    pub description_span: Span,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
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
    Experimental,
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
    SuspendsFiber,
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
    PhpstanType,
    ImportType,
    PsalmImportType,
    PhpstanImportType,
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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
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
    #[must_use]
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
    #[must_use]
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

    #[must_use]
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
    T: AsRef<[u8]>,
{
    fn from(value: T) -> Self {
        let lowered = value.as_ref().to_ascii_lowercase();
        match lowered.as_slice() {
            b"abstract" => TagKind::Abstract,
            b"access" => TagKind::Access,
            b"author" => TagKind::Author,
            b"category" => TagKind::Category,
            b"copyright" => TagKind::Copyright,
            b"deprecated" => TagKind::Deprecated,
            b"example" => TagKind::Example,
            b"final" => TagKind::Final,
            b"filesource" => TagKind::FileSource,
            b"global" => TagKind::Global,
            b"ignore" => TagKind::Ignore,
            b"internal" => TagKind::Internal,
            b"license" => TagKind::License,
            b"link" => TagKind::Link,
            b"method" => TagKind::Method,
            b"mixin" => TagKind::Mixin,
            b"name" => TagKind::Name,
            b"package" => TagKind::Package,
            b"param" => TagKind::Param,
            b"property" => TagKind::Property,
            b"property-read" => TagKind::PropertyRead,
            b"propertyread" => TagKind::PropertyRead,
            b"property-write" => TagKind::PropertyWrite,
            b"propertywrite" => TagKind::PropertyWrite,
            b"sealproperties" => TagKind::SealProperties,
            b"seal-properties" => TagKind::SealProperties,
            b"nosealproperties" => TagKind::NoSealProperties,
            b"no-seal-properties" => TagKind::NoSealProperties,
            b"sealmethods" => TagKind::SealMethods,
            b"seal-methods" => TagKind::SealMethods,
            b"nosealmethods" => TagKind::NoSealMethods,
            b"no-seal-methods" => TagKind::NoSealMethods,
            b"readonly" => TagKind::ReadOnly,
            b"nonamedarguments" => TagKind::NoNamedArguments,
            b"no-named-arguments" => TagKind::NoNamedArguments,
            b"api" => TagKind::Api,
            b"psalm-api" | b"psalmapi" => TagKind::PsalmApi,
            b"experimental" => TagKind::Experimental,
            b"psalm-inheritors" | b"psalminheritors" => TagKind::PsalmInheritors,
            b"inheritors" => TagKind::Inheritors,
            b"return" => TagKind::Return,
            b"see" => TagKind::See,
            b"since" => TagKind::Since,
            b"static" => TagKind::Static,
            b"staticvar" => TagKind::StaticVar,
            b"static-var" => TagKind::StaticVar,
            b"subpackage" => TagKind::SubPackage,
            b"sub-package" => TagKind::SubPackage,
            b"todo" => TagKind::Todo,
            b"tutorial" => TagKind::Tutorial,
            b"uses" => TagKind::Uses,
            b"var" => TagKind::Var,
            b"throws" => TagKind::Throws,
            b"version" => TagKind::Version,
            b"assert" => TagKind::Assert,
            b"assert-if-true" | b"assertiftrue" => TagKind::AssertIfTrue,
            b"assert-if-false" | b"assertiffalse" => TagKind::AssertIfFalse,
            b"param-later-invoked-callable" => TagKind::ParamLaterInvokedCallable,
            b"paramlaterinvokedcallable" => TagKind::ParamLaterInvokedCallable,
            b"param-immediately-invoked-callable" => TagKind::ParamImmediatelyInvokedCallable,
            b"paramimmediatelyinvokedcallable" => TagKind::ParamImmediatelyInvokedCallable,
            b"param-closure-this" => TagKind::ParamClosureThis,
            b"paramclosurethis" => TagKind::ParamClosureThis,
            b"extends" => TagKind::Extends,
            b"template-extends" | b"templateextends" => TagKind::TemplateExtends,
            b"implements" => TagKind::Implements,
            b"template-implements" | b"templateimplements" => TagKind::TemplateImplements,
            b"use" => TagKind::Use,
            b"template-use" | b"templateuse" => TagKind::TemplateUse,
            b"not-deprecated" | b"notdeprecated" => TagKind::NotDeprecated,
            b"phpstan-impure" | b"phpstanimpure" => TagKind::PhpstanImpure,
            b"phpstan-pure" | b"phpstanpure" => TagKind::PhpstanPure,
            b"pure" => TagKind::Pure,
            b"immutable" => TagKind::Immutable,
            b"inheritdoc" | b"inheritdocs" | b"inherit-doc" | b"inherit-docs" => TagKind::InheritDoc,
            b"param-out" => TagKind::ParamOut,
            b"psalm-param-out" => TagKind::PsalmParamOut,
            b"consistentconstructor" | b"consistent-constructor" => TagKind::ConsistentConstructor,
            b"psalmconsistentconstructor" | b"psalm-consistent-constructor" => TagKind::PsalmConsistentConstructor,
            b"psalmconsistenttemplates" | b"psalm-consistent-templates" => TagKind::PsalmConsistentTemplates,
            b"psalm-var" => TagKind::PsalmVar,
            b"psalm-param" => TagKind::PsalmParam,
            b"psalm-return" => TagKind::PsalmReturn,
            b"psalm-property" => TagKind::PsalmProperty,
            b"psalm-property-read" => TagKind::PsalmPropertyRead,
            b"psalm-propertyread" => TagKind::PsalmPropertyRead,
            b"psalm-property-write" => TagKind::PsalmPropertyWrite,
            b"psalm-propertywrite" => TagKind::PsalmPropertyWrite,
            b"psalm-method" => TagKind::PsalmMethod,
            b"psalm-ignore-var" => TagKind::PsalmIgnoreVar,
            b"psalmignorevar" => TagKind::PsalmIgnoreVar,
            b"psalm-suppress" => TagKind::PsalmSuppress,
            b"psalm-assert" => TagKind::PsalmAssert,
            b"psalm-assert-if-true" | b"psalmassertiftrue" => TagKind::PsalmAssertIfTrue,
            b"psalm-assert-if-false" | b"psalmassertiffalse" => TagKind::PsalmAssertIfFalse,
            b"psalm-if-this-is" | b"psalmifthisis" => TagKind::PsalmIfThisIs,
            b"psalm-this-out" | b"psalmthisout" => TagKind::PsalmThisOut,
            b"ignore-nullable-return" | b"ignorenullablereturn" => TagKind::IgnoreNullableReturn,
            b"ignore-falsable-return" | b"ignorefalsablereturn" => TagKind::IgnoreFalsableReturn,
            b"psalm-ignore-nullable-return" | b"psalmignorenullablereturn" => TagKind::PsalmIgnoreNullableReturn,
            b"psalm-ignore-falsable-return" | b"psalmignorefalsablereturn" => TagKind::PsalmIgnoreFalsableReturn,
            b"psalm-seal-properties" => TagKind::PsalmSealProperties,
            b"psalmsealproperties" => TagKind::PsalmSealProperties,
            b"psalm-no-seal-properties" => TagKind::PsalmNoSealProperties,
            b"psalmnosealproperties" => TagKind::PsalmNoSealProperties,
            b"psalm-seal-methods" => TagKind::PsalmSealMethods,
            b"psalmsealmethods" => TagKind::PsalmSealMethods,
            b"psalm-no-seal-methods" => TagKind::PsalmNoSealMethods,
            b"psalmnosealmethods" => TagKind::PsalmNoSealMethods,
            b"psalm-internal" => TagKind::PsalmInternal,
            b"psalm-readonly" => TagKind::PsalmReadOnly,
            b"psalm-mutation-free" | b"psalmmutationfree" => TagKind::PsalmMutationFree,
            b"psalm-external-mutation-free" | b"psalmexternalmutationfree" => TagKind::PsalmExternalMutationFree,
            b"mutation-free" | b"mutationfree" => TagKind::MutationFree,
            b"external-mutation-free" | b"externalmutationfree" => TagKind::ExternalMutationFree,
            b"suspends-fiber" | b"suspendsfiber" => TagKind::SuspendsFiber,
            b"psalm-immutable" => TagKind::PsalmImmutable,
            b"psalm-pure" => TagKind::PsalmPure,
            b"psalm-allow-private-mutation" => TagKind::PsalmAllowPrivateMutation,
            b"psalmallowprivatemutation" => TagKind::PsalmAllowPrivateMutation,
            b"psalm-readonly-allow-private-mutation" => TagKind::PsalmReadOnlyAllowPrivateMutation,
            b"psalmreadonlyallowprivatemutation" => TagKind::PsalmReadOnlyAllowPrivateMutation,
            b"psalm-trace" => TagKind::PsalmTrace,
            b"psalm-check-type" => TagKind::PsalmCheckType,
            b"psalmchecktype" => TagKind::PsalmCheckType,
            b"psalm-check-type-exact" => TagKind::PsalmCheckTypeExact,
            b"psalmchecktypeexact" => TagKind::PsalmCheckTypeExact,
            b"psalm-taint-source" => TagKind::PsalmTaintSource,
            b"psalmtaintsource" => TagKind::PsalmTaintSource,
            b"psalm-taint-sink" => TagKind::PsalmTaintSink,
            b"psalmtaintsink" => TagKind::PsalmTaintSink,
            b"psalm-taint-escape" => TagKind::PsalmTaintEscape,
            b"psalmtaintescape" => TagKind::PsalmTaintEscape,
            b"psalm-taint-unescape" => TagKind::PsalmTaintUnescape,
            b"psalmtaintunescape" => TagKind::PsalmTaintUnescape,
            b"psalm-taint-specialize" => TagKind::PsalmTaintSpecialize,
            b"psalmtaintspecialize" => TagKind::PsalmTaintSpecialize,
            b"psalm-flow" => TagKind::PsalmFlow,
            b"psalmflow" => TagKind::PsalmFlow,
            b"psalm-require-extends" => TagKind::PsalmRequireExtends,
            b"psalmrequireextends" => TagKind::PsalmRequireExtends,
            b"psalm-require-implements" => TagKind::PsalmRequireImplements,
            b"psalmrequireimplements" => TagKind::PsalmRequireImplements,
            b"psalm-ignore-variable-property" => TagKind::PsalmIgnoreVariableProperty,
            b"psalmignorevariableproperty" => TagKind::PsalmIgnoreVariableProperty,
            b"psalm-ignore-variable-method" => TagKind::PsalmIgnoreVariableMethod,
            b"psalmignorevariablemethod" => TagKind::PsalmIgnoreVariableMethod,
            b"psalm-yield" => TagKind::PsalmYield,
            b"phpstan-assert" => TagKind::PhpstanAssert,
            b"phpstan-assert-if-true" => TagKind::PhpstanAssertIfTrue,
            b"phpstan-assert-if-false" => TagKind::PhpstanAssertIfFalse,
            b"phpstan-self-out" | b"phpstanselfout" => TagKind::PhpstanSelfOut,
            b"phpstan-this-out" | b"phpstanthisout" => TagKind::PhpstanThisOut,
            b"phpstan-require-extends" | b"phpstanrequireextends" => TagKind::PhpstanRequireExtends,
            b"phpstan-require-implements" | b"phpstanrequireimplements" => TagKind::PhpstanRequireImplements,
            b"template" => TagKind::Template,
            b"template-invariant" | b"templateinvariant" => TagKind::TemplateInvariant,
            b"template-covariant" | b"templatecovariant" => TagKind::TemplateCovariant,
            b"template-contravariant" | b"templatecontravariant" => TagKind::TemplateContravariant,
            b"psalm-template" | b"psalmtemplate" => TagKind::PsalmTemplate,
            b"psalm-template-invariant" | b"psalmtemplateinvariant" => TagKind::PsalmTemplateInvariant,
            b"psalm-template-covariant" | b"psalmtemplatecovariant" => TagKind::PsalmTemplateCovariant,
            b"psalm-template-contravariant" | b"psalmtemplatecontravariant" => TagKind::PsalmTemplateContravariant,
            b"phpstan-template" | b"phpstantemplate" => TagKind::PhpstanTemplate,
            b"phpstan-template-invariant" | b"phpstantemplateinvariant" => TagKind::PhpstanTemplateInvariant,
            b"phpstan-template-covariant" | b"phpstantemplatecovariant" => TagKind::PhpstanTemplateCovariant,
            b"phpstan-template-contravariant" | b"phpstantemplatecontravariant" => {
                TagKind::PhpstanTemplateContravariant
            }
            b"phpstan-param" => TagKind::PhpstanParam,
            b"phpstan-return" => TagKind::PhpstanReturn,
            b"phpstan-var" => TagKind::PhpstanVar,
            b"phpstan-readonly" => TagKind::PhpstanReadOnly,
            b"phpstan-immutable" => TagKind::PhpstanImmutable,
            b"enuminterface" | b"enum-interface" => TagKind::EnumInterface,
            b"mago-unchecked" | b"magounchecked" => TagKind::MagoUnchecked,
            b"unchecked" => TagKind::Unchecked,
            b"type" => TagKind::Type,
            b"phpstan-type" | b"phpstantype" => TagKind::PhpstanType,
            b"psalm-type" | b"psalmtype" => TagKind::PsalmType,
            b"import-type" | b"importtype" => TagKind::ImportType,
            b"phpstan-import-type" | b"phpstanimporttype" => TagKind::PhpstanImportType,
            b"psalm-import-type" | b"psalmimporttype" => TagKind::PsalmImportType,
            b"require-implements" | b"requireimplements" => TagKind::RequireImplements,
            b"require-extends" | b"requireextends" => TagKind::RequireExtends,
            b"self-out" | b"selfout" => TagKind::SelfOut,
            b"this-out" | b"thisout" => TagKind::ThisOut,
            b"where" => TagKind::Where,
            b"must-use" | b"mustuse" => TagKind::MustUse,
            _ => TagKind::Other,
        }
    }
}

impl TagVendor {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Mago => "mago",
            Self::Phpstan => "phpstan",
            Self::Psalm => "psalm",
        }
    }
}
