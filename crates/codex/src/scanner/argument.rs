use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::metadata::argument::ArgumentMetadata;
use crate::metadata::ttype::TypeMetadata;
use crate::scanner::Context;
use crate::scanner::inference::infer;

#[inline]
pub fn scan_argument_list(argument_list: &ArgumentList, context: &mut Context<'_>) -> Vec<ArgumentMetadata> {
    let mut arguments = vec![];
    for argument in argument_list.arguments.iter() {
        let metadata = match argument {
            Argument::Positional(positional_argument) => ArgumentMetadata::new_positional(positional_argument.span())
                .with_variadic(positional_argument.ellipsis.is_some())
                .with_inferred_type(infer(context.interner, context.resolved_names, &positional_argument.value).map(
                    |u| {
                        let mut type_metadata = TypeMetadata::new(u, positional_argument.value.span());
                        type_metadata.inferred = true;
                        type_metadata
                    },
                )),
            Argument::Named(named_argument) => {
                ArgumentMetadata::new_named(named_argument.name.value, named_argument.span()).with_inferred_type(
                    infer(context.interner, context.resolved_names, &named_argument.value).map(|u| {
                        let mut type_metadata = TypeMetadata::new(u, named_argument.value.span());
                        type_metadata.inferred = true;
                        type_metadata
                    }),
                )
            }
        };

        arguments.push(metadata);
    }

    arguments
}
