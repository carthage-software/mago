use mago_database::matcher::ExclusionMatcher;
use mago_syntax::ast::NodeKind;

use crate::integration::Integration;
use crate::integration::IntegrationSet;
use crate::rule::AnyRule;
use crate::settings::Settings;

#[derive(Debug, Clone)]
#[allow(clippy::field_scoped_visibility_modifiers)]
pub struct RuleRegistry {
    pub(crate) only: Option<Vec<String>>,
    integrations: IntegrationSet,
    rules: Vec<AnyRule>,
    rule_excludes: Vec<ExclusionMatcher<String>>,
    by_kind: Vec<Box<[usize]>>,
}

impl RuleRegistry {
    /// Builds a new `RuleRegistry` from settings.
    ///
    /// # Arguments
    ///
    /// * `only` - If `Some`, only builds rules whose codes are in this list.
    pub fn build(settings: &Settings, only: Option<&[String]>, include_disabled: bool) -> Self {
        let integrations = settings.integrations;
        let only = only.map(<[std::string::String]>::to_vec);

        let rules_with_excludes = AnyRule::get_all_for(settings, only.as_deref(), include_disabled || only.is_some());
        if let Some(only) = &only
            && rules_with_excludes.is_empty()
        {
            tracing::warn!("No rules found for the specified 'only' filter: {:?}", only);
        }

        let (rules, rule_exclude_patterns): (Vec<AnyRule>, Vec<Vec<String>>) = rules_with_excludes.into_iter().unzip();

        let rule_excludes: Vec<ExclusionMatcher<String>> = rule_exclude_patterns
            .into_iter()
            .enumerate()
            .filter_map(|(idx, patterns)| match ExclusionMatcher::compile(patterns, settings.glob) {
                Ok(matcher) => Some(matcher),
                Err(err) => {
                    tracing::error!(
                        "Failed to compile exclude patterns for rule `{}`: {err}. Patterns will be ignored.",
                        rules[idx].code()
                    );

                    None
                }
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

        Self { only, integrations, rules, rule_excludes, by_kind }
    }

    /// Checks if a specific rule is enabled in the registry.
    #[inline]
    #[must_use]
    pub fn is_rule_enabled(&self, code: &str) -> bool {
        self.rules.iter().any(|r| r.code() == code)
    }

    /// Checks if a specific integration is enabled in the registry.
    #[inline]
    #[must_use]
    pub fn is_integration_enabled(&self, name: Integration) -> bool {
        self.integrations.contains(name)
    }

    #[inline]
    #[must_use]
    pub fn integrations(&self) -> IntegrationSet {
        self.integrations
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
