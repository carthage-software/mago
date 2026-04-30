<?php

namespace Fixture;

// With `allow_implicit_pipe_callable_types` enabled, an arrow function used
// directly as the right-hand side of a pipe is exempt from the missing
// parameter / return type checks. The pipe operand carries enough type
// information to derive the parameter type, so requiring a hint is noise.
"foobar" |> (static fn($p) => $p);
