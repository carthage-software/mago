<?php

declare(strict_types=1);

function write(string $message, string ...$args): void
{
    echo $message, count($args);
}

function t(): void
{
    write('Hi', 'X', args: 'weekend');
    // @mago-expect analysis:duplicate-named-argument
    write('Hi', args: 'X', args: 'Y');
}
