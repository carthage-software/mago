use mago_syntax::ast::Argument;
use mago_syntax::ast::Expression;

use crate::invocation::InvocationArgumentsSource;

/// Retrieves an argument from the invocation arguments source based on the index or name.
///
/// # Arguments
///
/// * `call_arguments` - The source of invocation arguments, which can be an argument list, pipe input, or none.
/// * `index` - The index of the positional argument to retrieve.
/// * `names` - A vector of names to match against named arguments.
pub(super) fn get_argument<'ast, 'arena>(
    call_arguments: InvocationArgumentsSource<'ast, 'arena>,
    index: usize,
    names: Vec<&'static str>,
) -> Option<&'ast Expression<'arena>> {
    match call_arguments {
        InvocationArgumentsSource::ArgumentList(argument_list) => {
            if let Some(Argument::Positional(argument)) = argument_list.arguments.get(index) {
                return Some(&argument.value);
            }

            for argument in argument_list.arguments.iter() {
                let Argument::Named(named_argument) = argument else {
                    continue;
                };

                if names.contains(&named_argument.name.value) {
                    return Some(&named_argument.value);
                }
            }

            None
        }
        InvocationArgumentsSource::PipeInput(pipe) => {
            if index == 0 {
                Some(pipe.input)
            } else {
                None
            }
        }
        InvocationArgumentsSource::None(_) => None,
    }
}
