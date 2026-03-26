<?php

/**
 * @template T
 *
 * @param (callable(): T|never) $f
 *
 * @return T|never
 */
function x(callable $f): mixed
{
    return $f();
}

echo
    x(function (): string {
        return 'hello, world';
    })
;

echo
    x(
        /**
         * @return never|string
         */
        function (): string {
            if (rand(0, 1) == 1) {
                return 'hi again!';
            }

            exit('goodbye!');
        },
    )
;

x(function (): never {
    exit('goodbye!');
});
