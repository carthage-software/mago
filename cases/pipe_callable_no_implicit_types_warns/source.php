<?php

namespace Fixture;

// With `allow_implicit_pipe_callable_types` left at its default of `false`
// and the arrow-function check enabled, an inline pipe callable still has
// to declare both its parameter and return types.
/**
 */
'foobar' |> (static fn($p) => $p);
