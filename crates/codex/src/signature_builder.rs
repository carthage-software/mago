use mago_atom::ascii_lowercase_atom;
use mago_atom::atom;
use mago_database::file::File;
use mago_fingerprint::FingerprintOptions;
use mago_fingerprint::Fingerprintable;
use mago_names::ResolvedNames;
use mago_span::HasSpan;
use mago_syntax::ast::*;
use mago_syntax::walker::MutWalker;

use crate::signature::DefSignatureNode;
use crate::signature::FileSignature;

/// Builds a `FileSignature` from a Program AST using the walker pattern.
///
/// # Arguments
///
/// * `file` - The file being analyzed (used for line/column calculation)
/// * `program` - The parsed program AST
/// * `resolved_names` - The resolved names for the program (needed for fingerprinting)
///
/// # Returns
///
/// A `FileSignature` containing all top-level definitions with their hashes and positions.
pub fn build_file_signature<'arena>(
    file: &File,
    program: &'arena Program<'arena>,
    resolved_names: &'arena ResolvedNames<'arena>,
) -> FileSignature {
    let mut builder = SignatureBuilder::new(file, resolved_names);
    builder.walk_program(program, &mut ());

    let hash = program.fingerprint(resolved_names, &builder.fingerprint_options);

    FileSignature { hash, ast_nodes: builder.ast_nodes }
}

/// Context for building file signatures while walking the AST.
struct SignatureBuilder<'file, 'arena> {
    file: &'file File,
    resolved_names: &'arena ResolvedNames<'arena>,
    fingerprint_options: FingerprintOptions<'static>,
    class_stack: Vec<DefSignatureNode>,
    ast_nodes: Vec<DefSignatureNode>,
}

impl<'file, 'arena> SignatureBuilder<'file, 'arena> {
    fn new(file: &'file File, resolved_names: &'arena ResolvedNames<'arena>) -> Self {
        Self {
            file,
            resolved_names,
            fingerprint_options: FingerprintOptions::default(),
            class_stack: Vec::new(),
            ast_nodes: Vec::new(),
        }
    }

    fn create_node(
        &self,
        name: &str,
        is_function: bool,
        is_constant: bool,
        is_property: bool,
        span: mago_span::Span,
        hash: u64,
    ) -> DefSignatureNode {
        let start_line = self.file.line_number(span.start.offset);
        let end_line = self.file.line_number(span.end.offset);
        let start_column = self.file.column_number(span.start.offset) as u16;
        let end_column = self.file.column_number(span.end.offset) as u16;

        let atom_name = match (is_constant, is_property) {
            (true, _) | (_, true) => atom(name),
            _ => ascii_lowercase_atom(name),
        };

        DefSignatureNode::new(
            atom_name,
            is_function,
            is_constant,
            span.start.offset,
            span.end.offset,
            start_line,
            end_line,
            start_column,
            end_column,
            hash,
        )
    }
}

impl<'ast, 'arena> MutWalker<'ast, 'arena, ()> for SignatureBuilder<'_, 'arena> {
    fn walk_in_class(&mut self, class: &'ast Class<'arena>, _context: &mut ()) {
        let span = class.span();
        let name = self.resolved_names.get(&class.name);
        let hash = class.fingerprint(self.resolved_names, &self.fingerprint_options);

        let node = self.create_node(name, false, false, false, span, hash);
        self.class_stack.push(node);
    }

    fn walk_out_class(&mut self, _class: &'ast Class<'arena>, _context: &mut ()) {
        if let Some(node) = self.class_stack.pop() {
            self.ast_nodes.push(node);
        }
    }

    fn walk_in_interface(&mut self, interface: &'ast Interface<'arena>, _context: &mut ()) {
        let span = interface.span();
        let name = self.resolved_names.get(&interface.name);
        let hash = interface.fingerprint(self.resolved_names, &self.fingerprint_options);

        let node = self.create_node(name, false, false, false, span, hash);
        self.class_stack.push(node);
    }

    fn walk_out_interface(&mut self, _interface: &'ast Interface<'arena>, _context: &mut ()) {
        if let Some(node) = self.class_stack.pop() {
            self.ast_nodes.push(node);
        }
    }

    fn walk_in_trait(&mut self, r#trait: &'ast Trait<'arena>, _context: &mut ()) {
        let span = r#trait.span();
        let name = self.resolved_names.get(&r#trait.name);
        let hash = r#trait.fingerprint(self.resolved_names, &self.fingerprint_options);

        let node = self.create_node(name, false, false, false, span, hash);
        self.class_stack.push(node);
    }

    fn walk_out_trait(&mut self, _trait: &'ast Trait<'arena>, _context: &mut ()) {
        if let Some(node) = self.class_stack.pop() {
            self.ast_nodes.push(node);
        }
    }

    fn walk_in_enum(&mut self, r#enum: &'ast Enum<'arena>, _context: &mut ()) {
        let span = r#enum.span();
        let name = self.resolved_names.get(&r#enum.name);
        let hash = r#enum.fingerprint(self.resolved_names, &self.fingerprint_options);

        let node = self.create_node(name, false, false, false, span, hash);
        self.class_stack.push(node);
    }

    fn walk_out_enum(&mut self, _enum: &'ast Enum<'arena>, _context: &mut ()) {
        if let Some(node) = self.class_stack.pop() {
            self.ast_nodes.push(node);
        }
    }

    fn walk_in_function(&mut self, function: &'ast Function<'arena>, _context: &mut ()) {
        let span = function.span();
        let name = self.resolved_names.get(&function.name);
        let hash = function.fingerprint(self.resolved_names, &self.fingerprint_options);

        let node = self.create_node(name, true, false, false, span, hash);
        self.ast_nodes.push(node);
    }

    fn walk_in_constant(&mut self, constant: &'ast Constant<'arena>, _context: &mut ()) {
        let span = constant.span();
        let hash = constant.fingerprint(self.resolved_names, &self.fingerprint_options);

        for item in constant.items.iter() {
            let name = item.name.value;
            let node = self.create_node(name, false, true, false, span, hash);
            self.ast_nodes.push(node);
        }
    }

    fn walk_in_method(&mut self, method: &'ast Method<'arena>, _context: &mut ()) {
        let span = method.span();
        let name = method.name.value;
        let hash = method.fingerprint(self.resolved_names, &self.fingerprint_options);

        let node = self.create_node(name, true, false, false, span, hash);

        // Add method to the current class if we're inside one
        if let Some(class_node) = self.class_stack.last_mut() {
            class_node.children.push(node);
        }
    }

    fn walk_in_property(&mut self, property: &'ast Property<'arena>, _context: &mut ()) {
        let span = property.span();
        let hash = property.fingerprint(self.resolved_names, &self.fingerprint_options);

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
            let node = self.create_node(var_name, false, false, true, span, hash);

            // Add property to the current class if we're inside one
            if let Some(class_node) = self.class_stack.last_mut() {
                class_node.children.push(node);
            }
        }
    }

    fn walk_in_class_like_constant(&mut self, constant: &'ast ClassLikeConstant<'arena>, _context: &mut ()) {
        let span = constant.span();
        let hash = constant.fingerprint(self.resolved_names, &self.fingerprint_options);

        // Add the first constant item to the current class
        if let Some(item) = constant.items.first() {
            let name = item.name.value;
            let node = self.create_node(name, false, true, false, span, hash);

            if let Some(class_node) = self.class_stack.last_mut() {
                class_node.children.push(node);
            }
        }
    }

    fn walk_in_enum_case(&mut self, case: &'ast EnumCase<'arena>, _context: &mut ()) {
        let span = case.span();
        let hash = case.fingerprint(self.resolved_names, &self.fingerprint_options);

        // Extract enum case name
        let name = match &case.item {
            EnumCaseItem::Unit(unit) => unit.name.value,
            EnumCaseItem::Backed(backed) => backed.name.value,
        };

        let node = self.create_node(name, false, true, false, span, hash);

        // Add enum case to the current enum if we're inside one
        if let Some(enum_node) = self.class_stack.last_mut() {
            enum_node.children.push(node);
        }
    }
}
