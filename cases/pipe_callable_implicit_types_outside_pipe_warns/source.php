<?php

namespace Fixture;

// `allow_implicit_pipe_callable_types` only relaxes the checks when the
// arrow function / closure is the RHS of a pipe. A bare arrow function
// assignment still trips the missing-type-hint warnings.
/**
 */
$f = static fn($p) => $p;

$f('noop');
