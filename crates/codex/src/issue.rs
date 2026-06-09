use strum::Display;
use strum::EnumString;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Display, Debug, PartialOrd, Ord, EnumString)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum ScanningIssueKind {
    MalformedDocblockComment,
    InvalidReturnTag,
    InvalidWhereTag,
    InvalidParamOutTag,
    InvalidParamTag,
    InvalidThrowsTag,
    InvalidPropertyTag,
    InvalidAssertionTag,
    InvalidVarTag,
    InvalidTemplateTag,
    InvalidTypeTag,
    InvalidUseTag,
    InvalidExtendsTag,
    InvalidImplementsTag,
    InvalidRequireExtendsTag,
    InvalidRequireImplementsTag,
    InvalidInheritorsTag,
    InvalidMixinTag,
    CircularTypeImport,
    PatchDuplicateTarget,
    PatchKindMismatch,
    PatchReadonlyMismatch,
    PatchHierarchyMismatch,
    PatchDeclaresTrait,
    PatchIntroducesNewSymbol,
    PatchIntroducesNewMethod,
    PatchIntroducesNewProperty,
    PatchIntroducesNewConstant,
    PatchEnumCasesIgnored,
    PatchPropertyStructuralMismatch,
    PatchConstantStructuralMismatch,
    PatchMethodStructuralMismatch,
    PatchFunctionParameterMismatch,
    PatchFunctionParameterNameMismatch,
}

impl From<ScanningIssueKind> for String {
    fn from(val: ScanningIssueKind) -> Self {
        val.to_string()
    }
}
