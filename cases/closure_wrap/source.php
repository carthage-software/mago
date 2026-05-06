<?php

declare(strict_types=1);

function consume_string(string $s): void
{
    echo $s;
}

/**
 * @template T
 *
 * @param (callable(): T) $callback
 *
 * @return T
 */
function run(callable $callback): mixed
{
    return $callback();
}

/**
 * @param (callable(): mixed) $callback
 */
function execute(callable $callback): void
{
    $callback();
}

consume_string(run(function () {
    return run(function () {
        return 'Hello, World!';
    });
}));

execute(function (): void {
    consume_string(run(function () {
        return 'Hello, World!';
    }));
});

execute(function (): void {
    consume_string('Hello, World!');
});

execute(function () {
    consume_string(run(function () {
        return 'Hello, World!';
    }));
});

execute(function () {
    consume_string('Hello, World!');
});

run(function () {
    execute(function () {
        return 'Hello, World!';
    });
});

run(function (): void {
    execute(function () {
        return 'Hello, World!';
    });
});

run(function () {
    execute(function (): string {
        return 'Hello, World!';
    });
});

run(function (): void {
    execute(function (): string {
        return 'Hello, World!';
    });
});

run(function () {
    execute(function (): void {
        echo 'Hello, World!';
    });
});

run(function (): void {
    execute(function (): void {
        echo 'Hello, World!';
    });
});
