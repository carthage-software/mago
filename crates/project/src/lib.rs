use serde::Deserialize;
use serde::Serialize;

use mago_interner::ThreadedInterner;
use mago_reflection::CodebaseReflection;

use crate::module::Module;

mod internal;

pub mod module;

/// A builder for incrementally constructing a [`Project`].
///
/// `ProjectBuilder` allows you to start from an existing codebase reflection
/// or from an empty state, and then add [`Module`]s one by one. When finished,
/// calling [`build`] returns a fully constructed [`Project`] with all module reflections
/// merged into a unified reflection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectBuilder {
    interner: ThreadedInterner,
    reflection: CodebaseReflection,
    modules: Vec<Module>,
}

impl ProjectBuilder {
    /// Creates a new empty `ProjectBuilder` using the provided interner.
    ///
    /// This builder starts with an empty set of modules and an empty reflection.
    ///
    /// # Arguments
    ///
    /// * `interner` - A reference to a [`ThreadedInterner`] used for string interning and name resolution.
    pub fn new(interner: ThreadedInterner) -> Self {
        Self { interner, reflection: CodebaseReflection::new(), modules: Vec::new() }
    }

    pub fn from_reflection(interner: ThreadedInterner, reflection: CodebaseReflection) -> Self {
        Self { interner, reflection, modules: Vec::new() }
    }

    /// Adds a module to the builder.
    ///
    /// If the provided module has reflection data, it is merged into the builder's
    /// accumulated reflection, which is used to populate additional reflection data.
    ///
    /// # Arguments
    ///
    /// * `module` - A [`Module`] to be added to the builder.
    pub fn add_module(&mut self, mut module: Module) {
        if let Some(reflection) = module.reflection.take() {
            self.reflection.merge(&self.interner, reflection);
        }

        self.modules.push(module);
    }

    /// Consumes the builder and constructs a [`Project`].
    ///
    /// This method merges all the reflection data collected from the added modules,
    /// optionally populating additional reflection for non-user-defined elements, and
    /// returns a fully built [`Project`].
    ///
    /// # Arguments
    ///
    /// * `populate_non_user_defined` - A boolean flag indicating whether to populate reflection
    ///   data for non-user-defined elements.
    pub fn build(mut self, populate_non_user_defined: bool) -> Project {
        internal::populator::populate(&self.interner, &mut self.reflection, populate_non_user_defined);

        Project { modules: self.modules, reflection: self.reflection }
    }
}

/// Represents a complete PHP code project.
///
/// A `Project` is composed of multiple modules along with a unified reflection of the codebase.
/// The reflection provides a merged view of code insights (such as classes and functions) extracted from each module.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Project {
    /// A list of modules that comprise the project.
    pub modules: Vec<Module>,
    /// The merged reflection of the project containing information aggregated from each module.
    pub reflection: CodebaseReflection,
}

impl Project {
    /// Creates a new `ProjectBuilder` to start building a project.
    pub fn builder(interner: ThreadedInterner) -> ProjectBuilder {
        ProjectBuilder::new(interner)
    }
}
