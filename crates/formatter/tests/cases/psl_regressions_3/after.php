<?php

Psl\invariant(
    Type\array_key()->matches($key), // @mago-expect analysis:redundant-type-comparison
    'Expected $key_func to return a value of type array-key, value of type (%s) returned.',
    gettype($key),
);
