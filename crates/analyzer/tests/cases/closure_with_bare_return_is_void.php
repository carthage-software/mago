<?php

declare(strict_types=1);

function declared_void(): string
{
    $callback = function (): void {
        return;
    };

    $callback();

    return 'after';
}

function undeclared(bool $flag): string
{
    $callback = function () use ($flag) {
        if ($flag) {
            return;
        }

        echo 'not flagged';
    };

    $callback();

    return 'after';
}

/** @throws RuntimeException */
function throws_only(): string
{
    $callback =
        /** @throws RuntimeException */
        function (): never {
            throw new RuntimeException('always');
        };

    $callback();

    // @mago-expect analysis:unevaluated-code
    return 'after';
}
