//! AST walkers for Twig templates.
//!
//! Provides a [`MutWalker`] (`&mut self`) trait, a [`Walker`] (`&self`) trait,
//! and a set of free `walk_*` / `walk_*_mut` standalone functions - one method
//! per AST node type, auto-generated from a single DSL description below.

#![allow(unused_variables, clippy::only_used_in_recursion, clippy::wildcard_imports)]

use crate::ast::*;

macro_rules! define_walk_body {
    ($walker:ident, $context:ident, $var_name:ident, $code:block) => {
        paste::paste! {
            $walker.[<walk_in_ $var_name>]($var_name, $context);
            $code
            $walker.[<walk_out_ $var_name>]($var_name, $context);
        }
    };
}

macro_rules! gen_mut_trait_methods {
    ('arena, $node_type:ty, $var_name:ident, $walker:ident, $context:ident, $ast:lifetime, $arena:lifetime, $code:block) => {
        paste::paste! {
            #[inline]
            fn [<walk_in_ $var_name>](&mut self, $var_name: & $ast $node_type<$arena>, context: &mut C) {}
            #[inline]
            fn [<walk_ $var_name>](&mut self, $var_name: & $ast $node_type<$arena>, $context: &mut C) {
                let $walker = self;
                define_walk_body!($walker, $context, $var_name, $code);
            }
            #[inline]
            fn [<walk_out_ $var_name>](&mut self, $var_name: & $ast $node_type<$arena>, context: &mut C) {}
        }
    };
    (_, $node_type:ty, $var_name:ident, $walker:ident, $context:ident, $ast:lifetime, $arena:lifetime, $code:block) => {
        paste::paste! {
            #[inline]
            fn [<walk_in_ $var_name>](&mut self, $var_name: & $ast $node_type, context: &mut C) {}
            #[inline]
            fn [<walk_ $var_name>](&mut self, $var_name: & $ast $node_type, $context: &mut C) {
                let $walker = self;
                define_walk_body!($walker, $context, $var_name, $code);
            }
            #[inline]
            fn [<walk_out_ $var_name>](&mut self, $var_name: & $ast $node_type, context: &mut C) {}
        }
    };
}

macro_rules! gen_const_trait_methods {
    ('arena, $node_type:ty, $var_name:ident, $walker:ident, $context:ident, $ast:lifetime, $arena:lifetime, $code:block) => {
        paste::paste! {
            #[inline]
            fn [<walk_in_ $var_name>](&self, $var_name: & $ast $node_type<$arena>, context: &mut C) {}
            #[inline]
            fn [<walk_ $var_name>](&self, $var_name: & $ast $node_type<$arena>, $context: &mut C) {
                let $walker = self;
                define_walk_body!($walker, $context, $var_name, $code);
            }
            #[inline]
            fn [<walk_out_ $var_name>](&self, $var_name: & $ast $node_type<$arena>, context: &mut C) {}
        }
    };
    (_, $node_type:ty, $var_name:ident, $walker:ident, $context:ident, $ast:lifetime, $arena:lifetime, $code:block) => {
        paste::paste! {
            #[inline]
            fn [<walk_in_ $var_name>](&self, $var_name: & $ast $node_type, context: &mut C) {}
            #[inline]
            fn [<walk_ $var_name>](&self, $var_name: & $ast $node_type, $context: &mut C) {
                let $walker = self;
                define_walk_body!($walker, $context, $var_name, $code);
            }
            #[inline]
            fn [<walk_out_ $var_name>](&self, $var_name: & $ast $node_type, context: &mut C) {}
        }
    };
}

macro_rules! gen_standalone_funcs {
    ('arena, $node_type:ty, $var_name:ident, $walker:ident, $context:ident, $ast:lifetime, $arena:lifetime, $code:block) => {
        paste::paste! {
            #[inline]
            pub fn [<walk_ $var_name _mut>]<$ast, $arena, W, C>($walker: &mut W, $var_name: & $ast $node_type<$arena>, $context: &mut C)
                where W: ?Sized + MutWalker<$ast, $arena, C>
            {
                define_walk_body!($walker, $context, $var_name, $code);
            }

            #[inline]
            pub fn [<walk_ $var_name>]<$ast, $arena, W, C>($walker: &W, $var_name: & $ast $node_type<$arena>, $context: &mut C)
                where W: ?Sized + Walker<$ast, $arena, C>
            {
                define_walk_body!($walker, $context, $var_name, $code);
            }
        }
    };
    (_, $node_type:ty, $var_name:ident, $walker:ident, $context:ident, $ast:lifetime, $arena:lifetime, $code:block) => {
        paste::paste! {
            #[inline]
            pub fn [<walk_ $var_name _mut>]<$ast, $arena, W, C>($walker: &mut W, $var_name: & $ast $node_type, $context: &mut C)
                where W: ?Sized + MutWalker<$ast, $arena, C>
            {
                define_walk_body!($walker, $context, $var_name, $code);
            }

            #[inline]
            pub fn [<walk_ $var_name>]<$ast, $arena, W, C>($walker: &W, $var_name: & $ast $node_type, $context: &mut C)
                where W: ?Sized + Walker<$ast, $arena, C>
            {
                define_walk_body!($walker, $context, $var_name, $code);
            }
        }
    };
}

macro_rules! generate_ast_walker {
    (
        using($walker:ident, $context:ident, $ast:lifetime, $arena:lifetime):
        $(
            $prefix:tt $node_type:ty as $var_name:ident => $code:block
        )*
    ) => {
        /// Mutable-`self` AST walker.
        pub trait MutWalker<$ast, $arena, C>: Sync + Send {
            $(
                gen_mut_trait_methods!($prefix, $node_type, $var_name, $walker, $context, $ast, $arena, $code);
            )*
        }

        /// Immutable-`self` AST walker.
        pub trait Walker<$ast, $arena, C>: Sync + Send {
            $(
                gen_const_trait_methods!($prefix, $node_type, $var_name, $walker, $context, $ast, $arena, $code);
            )*
        }

        $(
            gen_standalone_funcs!($prefix, $node_type, $var_name, $walker, $context, $ast, $arena, $code);
        )*
    }
}

generate_ast_walker! {
    using(walker, context, 'ast, 'arena):

    'arena Template as template => {
        for statement in &template.statements {
            walker.walk_statement(statement, context);
        }
    }

    'arena Statement as statement => {
        match statement {
            Statement::Text(n) => walker.walk_text(n, context),
            Statement::Print(n) => walker.walk_print(n, context),
            Statement::Verbatim(n) => walker.walk_verbatim(n, context),
            Statement::If(n) => walker.walk_if(n, context),
            Statement::For(n) => walker.walk_for(n, context),
            Statement::Set(n) => walker.walk_set(n, context),
            Statement::Block(n) => walker.walk_block(n, context),
            Statement::Extends(n) => walker.walk_extends(n, context),
            Statement::Use(n) => walker.walk_use(n, context),
            Statement::Include(n) => walker.walk_include(n, context),
            Statement::Embed(n) => walker.walk_embed(n, context),
            Statement::Import(n) => walker.walk_import(n, context),
            Statement::From(n) => walker.walk_from(n, context),
            Statement::Macro(n) => walker.walk_macro(n, context),
            Statement::With(n) => walker.walk_with(n, context),
            Statement::Apply(n) => walker.walk_apply(n, context),
            Statement::Autoescape(n) => walker.walk_autoescape(n, context),
            Statement::Sandbox(n) => walker.walk_sandbox(n, context),
            Statement::Deprecated(n) => walker.walk_deprecated(n, context),
            Statement::Do(n) => walker.walk_do(n, context),
            Statement::Flush(n) => walker.walk_flush(n, context),
            Statement::Guard(n) => walker.walk_guard(n, context),
            Statement::Cache(n) => walker.walk_cache(n, context),
            Statement::Types(n) => walker.walk_types(n, context),
            Statement::Unknown(n) => walker.walk_unknown(n, context),
        }
    }

    'arena Expression as expression => {
        match expression {
            Expression::Name(n) => walker.walk_name(n, context),
            Expression::Number(n) => walker.walk_number(n, context),
            Expression::String(n) => walker.walk_string_literal(n, context),
            Expression::InterpolatedString(n) => walker.walk_interpolated_string(n, context),
            Expression::Bool(n) => walker.walk_bool(n, context),
            Expression::Null(n) => walker.walk_null(n, context),
            Expression::Array(n) => walker.walk_array(n, context),
            Expression::HashMap(n) => walker.walk_hash_map(n, context),
            Expression::Unary(n) => walker.walk_unary(n, context),
            Expression::Binary(n) => walker.walk_binary(n, context),
            Expression::Conditional(n) => walker.walk_conditional(n, context),
            Expression::GetAttribute(n) => walker.walk_get_attribute(n, context),
            Expression::GetItem(n) => walker.walk_get_item(n, context),
            Expression::Slice(n) => walker.walk_slice(n, context),
            Expression::Call(n) => walker.walk_call(n, context),
            Expression::MethodCall(n) => walker.walk_method_call(n, context),
            Expression::Filter(n) => walker.walk_filter(n, context),
            Expression::Test(n) => walker.walk_test(n, context),
            Expression::Parenthesized(n) => walker.walk_parenthesized(n, context),
            Expression::ArrowFunction(n) => walker.walk_arrow_function(n, context),
        }
    }

    'arena Keyword as keyword => {}
    'arena Identifier as identifier => {}

    'arena Text as text => {}

    'arena Print as print => {
        walker.walk_expression(&print.expression, context);
    }

    'arena Verbatim as verbatim => {}

    'arena If as r#if => {
        for branch in &r#if.branches {
            walker.walk_if_branch(branch, context);
        }
        if let Some(else_branch) = &r#if.else_branch {
            walker.walk_else_branch(else_branch, context);
        }
    }

    'arena IfBranch as if_branch => {
        walker.walk_expression(&if_branch.condition, context);
        for statement in &if_branch.body {
            walker.walk_statement(statement, context);
        }
    }

    'arena ElseBranch as else_branch => {
        for statement in &else_branch.body {
            walker.walk_statement(statement, context);
        }
    }

    'arena For as r#for => {
        walker.walk_expression(&r#for.sequence, context);
        if let Some(clause) = &r#for.if_clause {
            walker.walk_for_if_clause(clause, context);
        }
        for statement in &r#for.body {
            walker.walk_statement(statement, context);
        }
        if let Some(else_branch) = &r#for.else_branch {
            walker.walk_else_branch(else_branch, context);
        }
    }

    'arena ForIfClause as for_if_clause => {
        walker.walk_expression(&for_if_clause.condition, context);
    }

    'arena Set as set => {
        walker.walk_set_body(&set.body, context);
    }

    'arena SetBody as set_body => {
        match set_body {
            SetBody::Inline(inline) => walker.walk_set_inline(inline, context),
            SetBody::Capture(capture) => walker.walk_set_capture(capture, context),
        }
    }

    'arena SetInline as set_inline => {
        for value in set_inline.values.iter() {
            walker.walk_expression(value, context);
        }
    }

    'arena SetCapture as set_capture => {
        for statement in &set_capture.body {
            walker.walk_statement(statement, context);
        }
    }

    'arena Block as block => {
        walker.walk_block_body(&block.body, context);
    }

    'arena BlockBody as block_body => {
        match block_body {
            BlockBody::Short(short) => walker.walk_block_short(short, context),
            BlockBody::Long(long) => walker.walk_block_long(long, context),
        }
    }

    'arena BlockShort as block_short => {
        walker.walk_expression(&block_short.expression, context);
    }

    'arena BlockLong as block_long => {
        for statement in &block_long.body {
            walker.walk_statement(statement, context);
        }
    }

    'arena Extends as extends => {
        walker.walk_expression(&extends.template, context);
    }

    'arena Use as r#use => {
        walker.walk_expression(&r#use.template, context);
        for alias in r#use.aliases.iter() {
            walker.walk_block_alias(alias, context);
        }
    }

    'arena BlockAlias as block_alias => {}

    'arena Include as include => {
        walker.walk_expression(&include.template, context);
        if let Some(clause) = &include.with_clause {
            walker.walk_with_expression_clause(clause, context);
        }
    }

    'arena WithExpressionClause as with_expression_clause => {
        walker.walk_expression(&with_expression_clause.variables, context);
    }

    'arena Embed as embed => {
        walker.walk_expression(&embed.template, context);
        if let Some(clause) = &embed.with_clause {
            walker.walk_with_expression_clause(clause, context);
        }
        for statement in &embed.body {
            walker.walk_statement(statement, context);
        }
    }

    'arena Import as import => {
        walker.walk_expression(&import.template, context);
    }

    'arena From as from => {
        walker.walk_expression(&from.template, context);
    }

    'arena Macro as r#macro => {
        for argument in r#macro.arguments.iter() {
            walker.walk_macro_argument(argument, context);
        }
        for statement in &r#macro.body {
            walker.walk_statement(statement, context);
        }
    }

    'arena MacroArgument as macro_argument => {
        if let Some(default) = &macro_argument.default {
            walker.walk_expression(default, context);
        }
    }

    'arena With as with => {
        if let Some(variables) = &with.variables {
            walker.walk_expression(variables, context);
        }
        for statement in &with.body {
            walker.walk_statement(statement, context);
        }
    }

    'arena Apply as apply => {
        for filter in apply.filters.iter() {
            walker.walk_filter_application(filter, context);
        }
        for statement in &apply.body {
            walker.walk_statement(statement, context);
        }
    }

    'arena FilterApplication as filter_application => {
        if let Some(list) = &filter_application.argument_list {
            walker.walk_argument_list(list, context);
        }
    }

    'arena ArgumentList as argument_list => {
        for argument in argument_list.arguments.iter() {
            walker.walk_argument(argument, context);
        }
    }

    'arena Argument as argument => {
        match argument {
            Argument::Positional(a) => walker.walk_positional_argument(a, context),
            Argument::Named(a) => walker.walk_named_argument(a, context),
        }
    }

    'arena PositionalArgument as positional_argument => {
        walker.walk_expression(positional_argument.value, context);
    }

    'arena NamedArgument as named_argument => {
        walker.walk_expression(named_argument.value, context);
    }

    'arena Autoescape as autoescape => {
        if let Some(strategy) = &autoescape.strategy {
            walker.walk_expression(strategy, context);
        }
        for statement in &autoescape.body {
            walker.walk_statement(statement, context);
        }
    }

    'arena Sandbox as sandbox => {
        for statement in &sandbox.body {
            walker.walk_statement(statement, context);
        }
    }

    'arena Deprecated as deprecated => {
        walker.walk_expression(&deprecated.message, context);
        for option in &deprecated.options {
            walker.walk_deprecated_option(option, context);
        }
    }

    'arena DeprecatedOption as deprecated_option => {
        walker.walk_expression(&deprecated_option.value, context);
    }

    'arena Do as r#do => {
        walker.walk_expression(&r#do.expression, context);
    }

    'arena Flush as flush => {}

    'arena Guard as guard => {
        for statement in &guard.body {
            walker.walk_statement(statement, context);
        }
        if let Some(else_branch) = &guard.else_branch {
            walker.walk_else_branch(else_branch, context);
        }
    }

    'arena Cache as cache => {
        walker.walk_expression(&cache.key, context);
        if let Some(option) = &cache.ttl {
            walker.walk_cache_option(option, context);
        }
        if let Some(option) = &cache.tags {
            walker.walk_cache_option(option, context);
        }
        for statement in &cache.body {
            walker.walk_statement(statement, context);
        }
    }

    'arena CacheOption as cache_option => {
        walker.walk_expression(&cache_option.value, context);
    }

    'arena Types as types => {
        walker.walk_expression(&types.mapping, context);
    }

    'arena Unknown as unknown => {}

    'arena Name as name => {}
    'arena Number as number => {}
    'arena StringLiteral as string_literal => {}

    'arena InterpolatedString as interpolated_string => {
        for part in &interpolated_string.parts {
            walker.walk_string_part(part, context);
        }
    }

    'arena StringPart as string_part => {
        match string_part {
            StringPart::Literal(literal) => walker.walk_interpolated_literal(literal, context),
            StringPart::Interpolation(interpolation) => walker.walk_interpolation(interpolation, context),
        }
    }

    'arena InterpolatedLiteral as interpolated_literal => {}

    'arena Interpolation as interpolation => {
        walker.walk_expression(interpolation.expression, context);
    }

    _ Bool as bool => {}
    _ Null as null => {}

    'arena Array as array => {
        for element in array.elements.iter() {
            walker.walk_array_element(element, context);
        }
    }

    'arena ArrayElement as array_element => {
        match array_element {
            ArrayElement::Value(e) => walker.walk_value_array_element(e, context),
            ArrayElement::Variadic(e) => walker.walk_variadic_array_element(e, context),
            ArrayElement::Missing(e) => walker.walk_missing_array_element(e, context),
        }
    }

    'arena ValueArrayElement as value_array_element => {
        walker.walk_expression(value_array_element.value, context);
    }

    'arena VariadicArrayElement as variadic_array_element => {
        walker.walk_expression(variadic_array_element.value, context);
    }

    _ MissingArrayElement as missing_array_element => {}

    'arena HashMap as hash_map => {
        for entry in hash_map.entries.iter() {
            walker.walk_hash_map_entry(entry, context);
        }
    }

    'arena HashMapEntry as hash_map_entry => {
        if let Some(key) = &hash_map_entry.key {
            walker.walk_expression(key, context);
        }
        walker.walk_expression(&hash_map_entry.value, context);
    }

    'arena Unary as unary => {
        walker.walk_expression(unary.operand, context);
    }

    'arena Binary as binary => {
        walker.walk_expression(binary.lhs, context);
        walker.walk_expression(binary.rhs, context);
    }

    'arena Conditional as conditional => {
        walker.walk_expression(conditional.condition, context);
        if let Some(then) = conditional.then {
            walker.walk_expression(then, context);
        }
        if let Some(r#else) = conditional.r#else {
            walker.walk_expression(r#else, context);
        }
    }

    'arena GetAttribute as get_attribute => {
        walker.walk_expression(get_attribute.object, context);
        walker.walk_expression(get_attribute.attribute, context);
    }

    'arena GetItem as get_item => {
        walker.walk_expression(get_item.object, context);
        walker.walk_expression(get_item.index, context);
    }

    'arena Slice as slice => {
        walker.walk_expression(slice.object, context);
        if let Some(start) = slice.start {
            walker.walk_expression(start, context);
        }
        if let Some(length) = slice.length {
            walker.walk_expression(length, context);
        }
    }

    'arena Call as call => {
        walker.walk_expression(call.callee, context);
        walker.walk_argument_list(&call.argument_list, context);
    }

    'arena MethodCall as method_call => {
        walker.walk_expression(method_call.object, context);
        walker.walk_argument_list(&method_call.argument_list, context);
    }

    'arena Filter as filter => {
        walker.walk_expression(filter.operand, context);
        if let Some(list) = &filter.argument_list {
            walker.walk_argument_list(list, context);
        }
    }

    'arena Test as test => {
        walker.walk_expression(test.operand, context);
        match &test.arguments {
            TestArguments::Parenthesised(list) => walker.walk_argument_list(list, context),
            TestArguments::Bare(e) => walker.walk_expression(e, context),
            TestArguments::None => {}
        }
    }

    'arena Parenthesized as parenthesized => {
        walker.walk_expression(parenthesized.inner, context);
    }

    'arena ArrowFunction as arrow_function => {
        walker.walk_expression(arrow_function.body, context);
    }

}
