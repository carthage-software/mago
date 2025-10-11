use crate::document::group::GroupIdentifier;
use crate::document::*;
use crate::internal::FormatterState;
use crate::internal::format::Format;
use crate::internal::format::block::block_is_empty;
use crate::internal::format::format_token;
use crate::internal::format::misc::print_modifiers;
use crate::internal::format::parameters::should_break_parameters;
use crate::internal::format::parameters::should_hug_the_only_parameter;
use crate::settings::*;
use crate::wrap;
use bumpalo::vec;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

#[derive(Debug, Clone, Copy)]
enum FunctionLikeBody<'arena> {
    Abstract(Span),
    Block(&'arena Block<'arena>),
}

#[derive(Debug, Clone, Copy)]
struct FunctionLikeSettings {
    pub inline_empty_braces: bool,
    pub brace_style: BraceStyle,
    pub space_before_params: bool,
}

#[derive(Debug, Clone, Copy)]
struct FunctionLikeParts<'arena> {
    pub attribute_lists: &'arena Sequence<'arena, AttributeList<'arena>>,
    pub modifiers: Option<&'arena Sequence<'arena, Modifier<'arena>>>,
    pub static_keyword: Option<&'arena Keyword<'arena>>,
    pub fn_or_function: &'arena Keyword<'arena>,
    pub ampersand: Option<Span>,
    pub name: Option<&'arena LocalIdentifier<'arena>>,
    pub parameter_list: &'arena FunctionLikeParameterList<'arena>,
    pub use_clause: Option<&'arena ClosureUseClause<'arena>>,
    pub return_type_hint: Option<&'arena FunctionLikeReturnTypeHint<'arena>>,
    pub body: FunctionLikeBody<'arena>,
}

impl<'arena> FunctionLikeParts<'arena> {
    pub fn for_closure(closure: &'arena Closure<'arena>) -> Self {
        Self {
            attribute_lists: &closure.attribute_lists,
            modifiers: None,
            static_keyword: closure.r#static.as_ref(),
            fn_or_function: &closure.function,
            ampersand: closure.ampersand,
            name: None,
            parameter_list: &closure.parameter_list,
            use_clause: closure.use_clause.as_ref(),
            return_type_hint: closure.return_type_hint.as_ref(),
            body: FunctionLikeBody::Block(&closure.body),
        }
    }

    pub fn for_function(function: &'arena Function<'arena>) -> Self {
        Self {
            attribute_lists: &function.attribute_lists,
            modifiers: None,
            static_keyword: None,
            fn_or_function: &function.function,
            ampersand: function.ampersand,
            name: Some(&function.name),
            parameter_list: &function.parameter_list,
            use_clause: None,
            return_type_hint: function.return_type_hint.as_ref(),
            body: FunctionLikeBody::Block(&function.body),
        }
    }

    pub fn for_method(method: &'arena Method<'arena>) -> Self {
        Self {
            attribute_lists: &method.attribute_lists,
            modifiers: Some(&method.modifiers),
            static_keyword: None,
            fn_or_function: &method.function,
            ampersand: method.ampersand,
            name: Some(&method.name),
            parameter_list: &method.parameter_list,
            use_clause: None,
            return_type_hint: method.return_type_hint.as_ref(),
            body: match &method.body {
                MethodBody::Abstract(body) => FunctionLikeBody::Abstract(body.semicolon),
                MethodBody::Concrete(block) => FunctionLikeBody::Block(block),
            },
        }
    }

    fn get_leading_comment_span(&self) -> Span {
        if let Some(modifiers) = self.modifiers {
            modifiers.first_span().unwrap_or(self.fn_or_function.span)
        } else if let Some(static_kw) = self.static_keyword {
            static_kw.span
        } else {
            self.fn_or_function.span
        }
    }

    fn format_attributes(&self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        let mut attributes = vec![in f.arena];
        for attribute_list in self.attribute_lists.iter() {
            attributes.push(attribute_list.format(f));
            attributes.push(Document::Line(Line::hard()));

            // Closures add BreakParent after attributes
            if self.static_keyword.is_some() || self.use_clause.is_some() {
                attributes.push(Document::BreakParent);
            }
        }

        Document::Group(Group::new(attributes))
    }

    fn should_use_inlined_braces(&self, f: &FormatterState<'_, 'arena>, settings: &FunctionLikeSettings) -> bool {
        match &self.body {
            FunctionLikeBody::Abstract(_) => false,
            FunctionLikeBody::Block(block) => {
                settings.inline_empty_braces && block_is_empty(f, &block.left_brace, &block.right_brace)
            }
        }
    }

    fn format_signature(
        &self,
        f: &mut FormatterState<'_, 'arena>,
        space_before_params: bool,
    ) -> (Document<'arena>, GroupIdentifier) {
        let mut signature = vec![in f.arena];

        // Add modifiers (for methods)
        if let Some(modifiers) = self.modifiers {
            let mut modifier_docs = print_modifiers(f, modifiers);
            if !modifier_docs.is_empty() {
                modifier_docs.push(Document::space());
                signature.extend(modifier_docs);
            }
        }

        // Add static keyword (for closures)
        if let Some(static_kw) = self.static_keyword {
            signature.push(static_kw.format(f));
            signature.push(Document::space());
        }

        // Add function keyword
        signature.push(self.fn_or_function.format(f));

        // Add space before params if needed (closures)
        if space_before_params {
            signature.push(Document::space());
        } else if self.name.is_some() {
            // Functions and methods always have space before name
            signature.push(Document::space());
        }

        // Add ampersand if present
        if self.ampersand.is_some() {
            signature.push(Document::String("&"));
        }

        // Add name if present (functions and methods)
        if let Some(name) = self.name {
            signature.push(name.format(f));
        }

        let signature_id = f.next_id();
        // Add parameter list directly - don't wrap in another group
        signature.push(Document::Group(
            Group::new(vec![
                in f.arena;
                self.parameter_list.format(f)
            ])
            .with_id(signature_id),
        ));

        // Add use clause (for closures)
        if let Some(use_clause) = self.use_clause {
            signature.push(Document::space());
            signature.push(use_clause.format(f));
        }

        // Add return type hint
        if let Some(return_type) = self.return_type_hint {
            signature.push(return_type.format(f));
        }

        (Document::Group(Group::new(signature)), signature_id)
    }

    fn format_brace_spacing(
        &self,
        f: &mut FormatterState<'_, 'arena>,
        settings: &FunctionLikeSettings,
        signature_id: GroupIdentifier,
        parameter_list_will_break: Option<bool>,
        inlined_braces: bool,
    ) -> Document<'arena> {
        match settings.brace_style {
            BraceStyle::SameLine => Document::space(),
            BraceStyle::NextLine => {
                if inlined_braces {
                    return Document::space();
                }

                if let Some(false) = parameter_list_will_break {
                    return Document::Line(Line::hard());
                }

                Document::IfBreak(
                    IfBreak::new(f.arena, Document::space(), Document::Line(Line::hard())).with_id(signature_id),
                )
            }
        }
    }

    fn format_body(
        &self,
        f: &mut FormatterState<'_, 'arena>,
        settings: &FunctionLikeSettings,
        signature_id: GroupIdentifier,
        parameter_list_will_break: Option<bool>,
    ) -> Document<'arena> {
        match self.body {
            FunctionLikeBody::Abstract(span) => format_token(f, span, ";"),
            FunctionLikeBody::Block(block) => {
                let inlined_braces = self.should_use_inlined_braces(f, settings);
                let spacing =
                    self.format_brace_spacing(f, settings, signature_id, parameter_list_will_break, inlined_braces);

                Document::Group(Group::new(vec![
                    in f.arena;
                    spacing,
                    block.format(f),
                ]))
            }
        }
    }

    pub fn format(&self, f: &mut FormatterState<'_, 'arena>, settings: FunctionLikeSettings) -> Document<'arena> {
        let attributes = self.format_attributes(f);
        let leading_comment_span = self.get_leading_comment_span();
        let leading_comments = f.print_leading_comments(leading_comment_span);

        let parameter_list_will_break = if self.parameter_list.parameters.is_empty() {
            if f.has_inner_comment(self.parameter_list.span()) { None } else { Some(false) }
        } else if should_break_parameters(f, self.parameter_list) {
            Some(true)
        } else if should_hug_the_only_parameter(f, self.parameter_list) {
            Some(false)
        } else {
            None
        };

        let (signature, signature_id) = self.format_signature(f, settings.space_before_params);
        let body = self.format_body(f, &settings, signature_id, parameter_list_will_break);

        Document::Group(Group::new(vec![
            in f.arena;
            attributes,
            leading_comments.unwrap_or_else(Document::empty),
            signature,
            body,
        ]))
    }
}

impl<'arena> Format<'arena> for Function<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Function, {
            let parts = FunctionLikeParts::for_function(self);
            let settings = FunctionLikeSettings {
                inline_empty_braces: f.settings.inline_empty_function_braces,
                brace_style: f.settings.function_brace_style,
                space_before_params: false,
            };

            parts.format(f, settings)
        })
    }
}

impl<'arena> Format<'arena> for Closure<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Closure, {
            let parts = FunctionLikeParts::for_closure(self);

            parts.format(
                f,
                FunctionLikeSettings {
                    inline_empty_braces: f.settings.inline_empty_closure_braces,
                    brace_style: f.settings.closure_brace_style,
                    space_before_params: f.settings.space_before_closure_parameter_list_parenthesis,
                },
            )
        })
    }
}

impl<'arena> Format<'arena> for ClosureUseClauseVariable<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, ClosureUseClauseVariable, {
            if self.ampersand.is_some() {
                Document::Group(Group::new(vec![in f.arena; Document::String("&"), self.variable.format(f)]))
            } else {
                self.variable.format(f)
            }
        })
    }
}

impl<'arena> Format<'arena> for ClosureUseClause<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, ClosureUseClause, {
            let mut contents = vec![in f.arena; self.r#use.format(f)];
            if f.settings.space_before_closure_use_clause_parenthesis {
                contents.push(Document::space());
            }

            contents.push(Document::String("("));

            let mut variables = vec![in f.arena];
            for variable in self.variables.iter() {
                variables.push(variable.format(f));
            }

            let mut inner_content = Document::join(f.arena, variables, Separator::CommaLine);
            inner_content.insert(0, Document::Line(Line::soft()));
            if f.settings.trailing_comma {
                inner_content.push(Document::IfBreak(IfBreak::then(f.arena, Document::String(","))));
            }

            contents.push(Document::Indent(inner_content));
            if let Some(comments) = f.print_dangling_comments(self.left_parenthesis.join(self.right_parenthesis), true)
            {
                contents.push(comments);
            } else {
                contents.push(Document::Line(Line::soft()));
            }

            contents.push(Document::String(")"));

            Document::Group(Group::new(contents))
        })
    }
}

impl<'arena> Format<'arena> for Method<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Method, {
            let parts = FunctionLikeParts::for_method(self);

            parts.format(
                f,
                FunctionLikeSettings {
                    inline_empty_braces: if self.name.value.eq_ignore_ascii_case("__construct") {
                        f.settings.inline_empty_constructor_braces
                    } else {
                        f.settings.inline_empty_method_braces
                    },
                    brace_style: f.settings.method_brace_style,
                    space_before_params: false,
                },
            )
        })
    }
}

impl<'arena> Format<'arena> for MethodBody<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, MethodBody, {
            match self {
                MethodBody::Abstract(b) => b.format(f),
                MethodBody::Concrete(b) => b.format(f),
            }
        })
    }
}

impl<'arena> Format<'arena> for MethodAbstractBody {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, MethodAbstractBody, { Document::String(";") })
    }
}
