use mago_fingerprint::FingerprintOptions;
use mago_fingerprint::Fingerprintable;
use mago_names::ResolvedNames;
use mago_syntax::cst::Class;
use mago_syntax::cst::ClassLikeConstant;
use mago_syntax::cst::Constant;
use mago_syntax::cst::Enum;
use mago_syntax::cst::EnumCase;
use mago_syntax::cst::EnumCaseItem;
use mago_syntax::cst::Function;
use mago_syntax::cst::Interface;
use mago_syntax::cst::Method;
use mago_syntax::cst::Program;
use mago_syntax::cst::Property;
use mago_syntax::cst::PropertyItem;
use mago_syntax::cst::Trait;
use mago_syntax::cst::Trivia;
use mago_syntax::walker::MutWalker;
use mago_word::ascii_lowercase_word;
use mago_word::word;

use crate::signature::DefSignatureNode;
use crate::signature::FileSignature;

/// Builds a `FileSignature` from a Program CST using the walker pattern.
///
/// # Arguments
///
/// * `program` - The parsed program CST
/// * `resolved_names` - The resolved names for the program (needed for fingerprinting)
///
/// # Returns
///
/// A `FileSignature` containing all top-level definitions with their hashes and positions.
#[must_use]
pub fn build_file_signature<'arena>(
    program: &'arena Program<'arena>,
    resolved_names: &'arena ResolvedNames<'arena>,
) -> FileSignature {
    let mut builder = SignatureBuilder::new(resolved_names, program.trivia.nodes);
    builder.walk_program(program, &mut ());

    let hash = program.fingerprint(resolved_names, &builder.fingerprint_options);

    FileSignature { hash, ast_nodes: builder.ast_nodes }
}

/// Context for building file signatures while walking the CST.
struct SignatureBuilder<'arena> {
    resolved_names: &'arena ResolvedNames<'arena>,
    fingerprint_options: FingerprintOptions<'arena>,
    sig_only_options: FingerprintOptions<'arena>,
    class_stack: Vec<DefSignatureNode>,
    ast_nodes: Vec<DefSignatureNode>,
}

impl<'arena> SignatureBuilder<'arena> {
    fn new(resolved_names: &'arena ResolvedNames<'arena>, trivia: &'arena [Trivia<'arena>]) -> Self {
        let fingerprint_options = FingerprintOptions::default().with_trivia_context(trivia);
        let sig_only_options = FingerprintOptions { signature_only: true, ..fingerprint_options };

        Self { resolved_names, fingerprint_options, sig_only_options, class_stack: Vec::new(), ast_nodes: Vec::new() }
    }

    fn create_node(
        &self,
        name: &[u8],
        is_function: bool,
        is_constant: bool,
        is_property: bool,
        hash: u64,
        signature_hash: u64,
    ) -> DefSignatureNode {
        let atom_name = match (is_constant, is_property) {
            (true, _) | (_, true) => word(name),
            _ => ascii_lowercase_word(name),
        };

        DefSignatureNode::new(atom_name, is_function, hash, signature_hash)
    }
}

impl<'ast, 'arena> MutWalker<'ast, 'arena, ()> for SignatureBuilder<'arena> {
    fn walk_in_class(&mut self, class: &'ast Class<'arena>, _context: &mut ()) {
        let name = self.resolved_names.get(&class.name);
        let hash = class.fingerprint(self.resolved_names, &self.fingerprint_options);
        let signature_hash = class.fingerprint(self.resolved_names, &self.sig_only_options);

        let node = self.create_node(name, false, false, false, hash, signature_hash);
        self.class_stack.push(node);
    }

    fn walk_out_class(&mut self, _class: &'ast Class<'arena>, _context: &mut ()) {
        if let Some(node) = self.class_stack.pop() {
            self.ast_nodes.push(node);
        }
    }

    fn walk_in_interface(&mut self, interface: &'ast Interface<'arena>, _context: &mut ()) {
        let name = self.resolved_names.get(&interface.name);
        let hash = interface.fingerprint(self.resolved_names, &self.fingerprint_options);
        let signature_hash = interface.fingerprint(self.resolved_names, &self.sig_only_options);

        let node = self.create_node(name, false, false, false, hash, signature_hash);
        self.class_stack.push(node);
    }

    fn walk_out_interface(&mut self, _interface: &'ast Interface<'arena>, _context: &mut ()) {
        if let Some(node) = self.class_stack.pop() {
            self.ast_nodes.push(node);
        }
    }

    fn walk_in_trait(&mut self, r#trait: &'ast Trait<'arena>, _context: &mut ()) {
        let name = self.resolved_names.get(&r#trait.name);
        let hash = r#trait.fingerprint(self.resolved_names, &self.fingerprint_options);
        let signature_hash = r#trait.fingerprint(self.resolved_names, &self.sig_only_options);

        let node = self.create_node(name, false, false, false, hash, signature_hash);
        self.class_stack.push(node);
    }

    fn walk_out_trait(&mut self, _trait: &'ast Trait<'arena>, _context: &mut ()) {
        if let Some(node) = self.class_stack.pop() {
            self.ast_nodes.push(node);
        }
    }

    fn walk_in_enum(&mut self, r#enum: &'ast Enum<'arena>, _context: &mut ()) {
        let name = self.resolved_names.get(&r#enum.name);
        let hash = r#enum.fingerprint(self.resolved_names, &self.fingerprint_options);
        let signature_hash = r#enum.fingerprint(self.resolved_names, &self.sig_only_options);

        let node = self.create_node(name, false, false, false, hash, signature_hash);
        self.class_stack.push(node);
    }

    fn walk_out_enum(&mut self, _enum: &'ast Enum<'arena>, _context: &mut ()) {
        if let Some(node) = self.class_stack.pop() {
            self.ast_nodes.push(node);
        }
    }

    fn walk_in_function(&mut self, function: &'ast Function<'arena>, _context: &mut ()) {
        let name = self.resolved_names.get(&function.name);
        let hash = function.fingerprint(self.resolved_names, &self.fingerprint_options);
        let signature_hash = function.fingerprint(self.resolved_names, &self.sig_only_options);

        let node = self.create_node(name, true, false, false, hash, signature_hash);
        self.ast_nodes.push(node);
    }

    fn walk_in_constant(&mut self, constant: &'ast Constant<'arena>, _context: &mut ()) {
        let hash = constant.fingerprint(self.resolved_names, &self.fingerprint_options);
        // Constants don't have bodies — signature_hash == hash
        let signature_hash = hash;

        for item in &constant.items {
            let name = item.name.value;
            let node = self.create_node(name, false, true, false, hash, signature_hash);
            self.ast_nodes.push(node);
        }
    }

    fn walk_in_method(&mut self, method: &'ast Method<'arena>, _context: &mut ()) {
        let name = method.name.value;
        let hash = method.fingerprint(self.resolved_names, &self.fingerprint_options);
        let signature_hash = method.fingerprint(self.resolved_names, &self.sig_only_options);

        let node = self.create_node(name, true, false, false, hash, signature_hash);

        // Add method to the current class if we're inside one
        if let Some(class_node) = self.class_stack.last_mut() {
            class_node.children.push(node);
        }
    }

    fn walk_in_property(&mut self, property: &'ast Property<'arena>, _context: &mut ()) {
        let hash = property.fingerprint(self.resolved_names, &self.fingerprint_options);
        // Properties don't have traditional "bodies" — signature_hash == hash
        let signature_hash = hash;

        // Extract the first property variable name
        let name = match property {
            Property::Plain(plain) => plain.items.first().map(|item| match item {
                PropertyItem::Abstract(abstract_item) => &abstract_item.variable,
                PropertyItem::Concrete(concrete_item) => &concrete_item.variable,
            }),
            Property::Hooked(hooked) => match &hooked.item {
                PropertyItem::Abstract(abstract_item) => Some(&abstract_item.variable),
                PropertyItem::Concrete(concrete_item) => Some(&concrete_item.variable),
            },
        };

        if let Some(var) = name {
            let var_name = var.name;
            let node = self.create_node(var_name, false, false, true, hash, signature_hash);

            // Add property to the current class if we're inside one
            if let Some(class_node) = self.class_stack.last_mut() {
                class_node.children.push(node);
            }
        }
    }

    fn walk_in_class_like_constant(&mut self, constant: &'ast ClassLikeConstant<'arena>, _context: &mut ()) {
        let hash = constant.fingerprint(self.resolved_names, &self.fingerprint_options);
        // Class constants don't have bodies — signature_hash == hash
        let signature_hash = hash;

        // Add the first constant item to the current class
        if let Some(item) = constant.items.first() {
            let name = item.name.value;
            let node = self.create_node(name, false, true, false, hash, signature_hash);

            if let Some(class_node) = self.class_stack.last_mut() {
                class_node.children.push(node);
            }
        }
    }

    fn walk_in_enum_case(&mut self, case: &'ast EnumCase<'arena>, _context: &mut ()) {
        let hash = case.fingerprint(self.resolved_names, &self.fingerprint_options);
        // Enum cases don't have bodies — signature_hash == hash
        let signature_hash = hash;

        // Extract enum case name
        let name = match &case.item {
            EnumCaseItem::Unit(unit) => unit.name.value,
            EnumCaseItem::Backed(backed) => backed.name.value,
        };

        let node = self.create_node(name, false, true, false, hash, signature_hash);

        // Add enum case to the current enum if we're inside one
        if let Some(enum_node) = self.class_stack.last_mut() {
            enum_node.children.push(node);
        }
    }
}
