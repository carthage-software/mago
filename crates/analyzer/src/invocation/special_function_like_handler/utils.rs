use mago_syntax::ast::Argument;
use mago_syntax::ast::Expression;

use crate::context::Context;
use crate::invocation::InvocationArgumentsSource;

/// Retrieves an argument from the invocation arguments source based on the index or name.
///
/// # Arguments
///
/// * `context` - The current context containing interner and other data.
/// * `call_arguments` - The source of invocation arguments, which can be an argument list, pipe input, slice, or none.
/// * `index` - The index of the positional argument to retrieve.
/// * `names` - A vector of names to match against named arguments.
pub(super) fn get_argument<'argument>(
    context: &Context<'_>,
    call_arguments: InvocationArgumentsSource<'argument>,
    index: usize,
    names: Vec<&'static str>,
) -> Option<&'argument Expression> {
    match call_arguments {
        InvocationArgumentsSource::ArgumentList(argument_list) => {
            if let Some(Argument::Positional(argument)) = argument_list.arguments.get(index) {
                return Some(&argument.value);
            }

            for argument in argument_list.arguments.iter() {
                let Argument::Named(named_argument) = argument else {
                    continue;
                };

                let name = context.interner.lookup(&named_argument.name.value);
                if names.contains(&name) {
                    return Some(&named_argument.value);
                }
            }

            None
        }
        InvocationArgumentsSource::PipeInput(pipe) => {
            if index == 0 {
                Some(&pipe.input)
            } else {
                None
            }
        }
        InvocationArgumentsSource::Slice(_) => None,
        InvocationArgumentsSource::None(_) => None,
    }
}
