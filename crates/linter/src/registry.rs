use mago_syntax::ast::NodeKind;

use crate::integration::Integration;
use crate::integration::IntegrationSet;
use crate::rule::AnyRule;
use crate::settings::Settings;

#[derive(Debug, Clone)]
pub struct RuleRegistry {
    integrations: IntegrationSet,
    rules: Vec<AnyRule>,
    by_kind: Vec<&'static [usize]>,
}

impl RuleRegistry {
    /// Builds a new `RuleRegistry` from settings.
    ///
    /// # Arguments
    ///
    /// * `only` - If `Some`, only builds rules whose codes are in this list.
    pub fn build(settings: &Settings, only: Option<&[String]>, include_disabled: bool) -> Self {
        let integrations = settings.integrations;
        let rules: Vec<AnyRule> = AnyRule::get_all_for(settings, only, include_disabled || only.is_some());

        let max_kind = u8::MAX as usize + 1;
        let mut temp: Vec<Vec<usize>> = vec![Vec::new(); max_kind];
        for (i, r) in rules.iter().enumerate() {
            for &k in r.targets() {
                temp[k as usize].push(i);
            }
        }

        let by_kind: Vec<&'static [usize]> =
            temp.into_iter().map(|v| Box::<[usize]>::leak(v.into_boxed_slice()) as &'static [usize]).collect();

        Self { integrations, rules, by_kind }
    }

    /// Checks if a specific rule is enabled in the registry.
    #[inline]
    pub fn is_rule_enabled(&self, code: &str) -> bool {
        self.rules.iter().any(|r| r.code() == code)
    }

    /// Checks if a specific integration is enabled in the registry.
    #[inline]
    pub fn is_integration_enabled(&self, name: Integration) -> bool {
        self.integrations.contains(name)
    }

    #[inline]
    pub fn integrations(&self) -> IntegrationSet {
        self.integrations
    }

    #[inline]
    pub fn rules(&self) -> &[AnyRule] {
        &self.rules
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    #[inline]
    pub fn for_kind(&self, kind: NodeKind) -> &'static [usize] {
        self.by_kind[kind as usize]
    }

    #[inline]
    pub fn rule(&self, idx: usize) -> &AnyRule {
        &self.rules[idx]
    }
}
