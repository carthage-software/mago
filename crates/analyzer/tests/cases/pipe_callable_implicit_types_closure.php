<?php

namespace Fixture;

// Same as `pipe_callable_implicit_types_arrow.php`, but for a long closure.
"foobar" |> (function ($p) {
    return $p;
});
