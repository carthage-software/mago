<?php

declare(strict_types=1);

function later_catch(): bool
{
    try {
        if (rand(0, 1)) {
            throw new InvalidArgumentException();
        }

        throw new RuntimeException();
    } catch (InvalidArgumentException) {
        echo 'invalid';
    } catch (RuntimeException) {
        $foo = true;
    }

    // @mago-expect analysis:possibly-undefined-variable
    return $foo;
}

function try_may_finish(): bool
{
    try {
        echo rand(0, 1);
    } catch (RuntimeException) {
        $foo = true;
    }

    // @mago-expect analysis:possibly-undefined-variable
    return $foo;
}
