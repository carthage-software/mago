use serde::Deserialize;
use serde::Serialize;
use strum::Display;
use strum::EnumString;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Display, Debug, PartialOrd, Ord, Serialize, Deserialize, EnumString)]
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
}

impl From<ScanningIssueKind> for String {
    fn from(val: ScanningIssueKind) -> Self {
        val.to_string()
    }
}
