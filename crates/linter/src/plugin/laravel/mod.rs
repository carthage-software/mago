use crate::plugin::laravel::rules::safety::no_request_all::NoRequestAllRule;

use crate::plugin::Plugin;
use crate::rule::Rule;

pub mod rules;

#[derive(Debug)]
pub struct LaravelPlugin;

impl Plugin for LaravelPlugin {
    fn get_name(&self) -> &'static str {
        "laravel"
    }

    fn is_enabled_by_default(&self) -> bool {
        false
    }

    fn get_rules(&self) -> Vec<Box<dyn Rule>> {
        vec![Box::new(NoRequestAllRule)]
    }
}
