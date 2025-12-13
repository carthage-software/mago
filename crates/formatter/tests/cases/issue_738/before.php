<?php

foo(
    $f,
    $a->then(
        fn($tag) => $tag
            ? [
                'arg1' => static::something($tag),
                'arg2' => static::somethingElse($tag),
            ]
            : null
    ),
);
