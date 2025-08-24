use ahash::HashMap;
use ahash::RandomState;
use indexmap::IndexMap;

use mago_atom::Atom;
use mago_span::Span;

use crate::misc::GenericParent;
use crate::ttype::union::TUnion;

pub mod inferred_type_replacer;
pub mod standin_type_replacer;
pub mod variance;

#[derive(Clone, Debug, Default)]
pub struct TemplateResult {
    pub template_types: IndexMap<Atom, Vec<(GenericParent, TUnion)>, RandomState>,
    pub lower_bounds: IndexMap<Atom, HashMap<GenericParent, Vec<TemplateBound>>, RandomState>,
    pub upper_bounds: IndexMap<Atom, HashMap<GenericParent, TemplateBound>, RandomState>,
    pub readonly: bool,
    pub upper_bounds_unintersectable_types: Vec<TUnion>,
}

impl TemplateResult {
    pub fn new(
        template_types: IndexMap<Atom, Vec<(GenericParent, TUnion)>, RandomState>,
        lower_bounds: IndexMap<Atom, HashMap<GenericParent, TUnion>, RandomState>,
    ) -> TemplateResult {
        let mut new_lower_bounds = IndexMap::with_hasher(RandomState::new());

        for (k, v) in lower_bounds {
            let mut th = HashMap::default();

            for (vk, vv) in v {
                th.insert(vk, vec![TemplateBound::new(vv, 0, None, None)]);
            }

            new_lower_bounds.insert(k, th);
        }

        TemplateResult {
            template_types,
            lower_bounds: new_lower_bounds,
            upper_bounds: IndexMap::with_hasher(RandomState::new()),
            readonly: false,
            upper_bounds_unintersectable_types: Vec::new(),
        }
    }

    pub fn has_template_types(&self) -> bool {
        !self.template_types.is_empty()
    }

    pub fn add_lower_bounds(&mut self, lower_bounds: IndexMap<Atom, HashMap<GenericParent, TUnion>, RandomState>) {
        for (k, v) in lower_bounds {
            let mut th = HashMap::default();

            for (vk, vv) in v {
                th.insert(vk, vec![TemplateBound::new(vv, 0, None, None)]);
            }

            self.lower_bounds.insert(k, th);
        }
    }

    pub fn add_lower_bound(&mut self, parameter_name: Atom, generic_parent: GenericParent, bound: TUnion) {
        let entry = self.lower_bounds.entry(parameter_name).or_default();

        entry.entry(generic_parent).or_default().push(TemplateBound::new(bound, 0, None, None));
    }

    pub fn add_template_type(&mut self, parameter_name: Atom, generic_parent: GenericParent, constraint: TUnion) {
        let entry = self.template_types.entry(parameter_name).or_default();
        entry.push((generic_parent, constraint));
    }

    pub fn add_upper_bound(&mut self, parameter_name: Atom, generic_parent: GenericParent, bound: TemplateBound) {
        let entry = self.upper_bounds.entry(parameter_name).or_default();
        entry.insert(generic_parent, bound);
    }

    pub fn add_upper_bound_unintersectable_type(&mut self, bound: TUnion) {
        self.upper_bounds_unintersectable_types.push(bound);
    }

    pub fn has_lower_bound(&self, parameter_name: &Atom, generic_parent: &GenericParent) -> bool {
        self.lower_bounds
            .get(parameter_name)
            .and_then(|bounds| bounds.get(generic_parent))
            .is_some_and(|bounds| !bounds.is_empty())
    }

    pub fn has_lower_bound_for_class_like(&self, parameter_name: &Atom, classlike_name: &Atom) -> bool {
        self.has_lower_bound(parameter_name, &GenericParent::ClassLike(*classlike_name))
    }

    pub fn get_lower_bounds_for_class_like(
        &self,
        parameter_name: &Atom,
        classlike_name: &Atom,
    ) -> Option<&Vec<TemplateBound>> {
        self.lower_bounds.get(parameter_name).and_then(|bounds| bounds.get(&GenericParent::ClassLike(*classlike_name)))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TemplateBound {
    pub bound_type: TUnion,
    pub appearance_depth: usize,
    pub argument_offset: Option<usize>,
    pub equality_bound_classlike: Option<Atom>,
    pub span: Option<Span>,
}

impl TemplateBound {
    pub fn new(
        bound_type: TUnion,
        appearance_depth: usize,
        argument_offset: Option<usize>,
        equality_bound_classlike: Option<Atom>,
    ) -> Self {
        Self { bound_type, appearance_depth, argument_offset, equality_bound_classlike, span: None }
    }

    pub fn of_type(bound_type: TUnion) -> Self {
        Self { bound_type, appearance_depth: 0, argument_offset: None, equality_bound_classlike: None, span: None }
    }
}
