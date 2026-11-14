use mago_database::matcher::ExclusionMatcher;
use mago_twig_syntax::cst::NodeKind;

use crate::rule::AnyRule;
use crate::settings::Settings;

#[derive(Debug, Clone)]
#[allow(clippy::field_scoped_visibility_modifiers)]
pub struct RuleRegistry {
    pub(crate) only: Option<Vec<String>>,
    rules: Vec<AnyRule>,
    rule_excludes: Vec<ExclusionMatcher<String>>,
    by_kind: Vec<Box<[usize]>>,
}

impl RuleRegistry {
    /// Builds a new `RuleRegistry` from settings.
    ///
    /// Rules that need a newer Twig version than the configured minimum are never loaded.
    ///
    /// # Arguments
    ///
    /// * `only` - If `Some`, only builds rules whose codes are in this list.
    /// * `include_disabled` - If `true`, includes rules that are disabled in the settings.
    pub fn build(settings: &Settings, only: Option<&[String]>, include_disabled: bool) -> Self {
        let only = only.map(<[std::string::String]>::to_vec);

        let rules_with_excludes = AnyRule::get_all_for(settings, only.as_deref(), include_disabled || only.is_some());

        let (rules, rule_exclude_patterns): (Vec<AnyRule>, Vec<Vec<String>>) = rules_with_excludes.into_iter().unzip();

        let rule_excludes: Vec<ExclusionMatcher<String>> = rule_exclude_patterns
            .into_iter()
            .enumerate()
            .map(|(idx, patterns)| {
                ExclusionMatcher::compile(patterns, settings.glob).unwrap_or_else(|err| {
                    tracing::error!(
                        "Failed to compile exclude patterns for rule `{}`: {err}. Patterns will be ignored.",
                        rules[idx].code()
                    );

                    ExclusionMatcher::compile(Vec::<String>::new(), settings.glob).unwrap_or_else(|_| unreachable!())
                })
            })
            .collect();

        let max_kind = u8::MAX as usize + 1;
        let mut temp: Vec<Vec<usize>> = vec![Vec::new(); max_kind];
        for (i, r) in rules.iter().enumerate() {
            for &k in r.targets() {
                temp[k as usize].push(i);
            }
        }

        let by_kind: Vec<Box<[usize]>> = temp.into_iter().map(|v| v.into_boxed_slice()).collect();

        Self { only, rules, rule_excludes, by_kind }
    }

    #[inline]
    #[must_use]
    pub fn rules(&self) -> &[AnyRule] {
        &self.rules
    }

    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    #[inline]
    #[must_use]
    pub fn for_kind(&self, kind: NodeKind) -> &[usize] {
        &self.by_kind[kind as usize]
    }

    #[inline]
    #[must_use]
    pub fn rule(&self, idx: usize) -> &AnyRule {
        &self.rules[idx]
    }

    #[inline]
    #[must_use]
    pub fn excludes_for(&self, idx: usize) -> &ExclusionMatcher<String> {
        &self.rule_excludes[idx]
    }
}
