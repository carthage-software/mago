<?php

declare(strict_types=1);

function write(string $message, string ...$args): void
{
    echo $message, count($args);
}

function t(): void
{
    write('Hi', other: 'fine');
    // @mago-expect analysis:invalid-argument
    write('Hi', other: 1);
    // @mago-expect analysis:invalid-argument
    write('Hi', first: 'fine', second: 2);
}
