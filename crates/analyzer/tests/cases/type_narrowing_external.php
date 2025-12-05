<?php

declare(strict_types=1);

function extractVersion(string $input): null|string
{
    $matches = [];
    if (preg_match('/v(\d+\.\d+\.\d+)/', $input, $matches)) {
        return $matches[1] ?? null;
    }

    return null;
}

function readFile(string $path): string
{
    $content = file_get_contents($path);
    if ($content === false) {
        return '';
    }

    return $content;
}

function processInput(int|string|null $input): string
{
    if ($input === null) {
        return 'null';
    }

    if (is_int($input)) {
        return 'int: ' . $input;
    }

    return 'string: ' . $input;
}

function test(): void
{
    $version = extractVersion('Release v1.2.3');
    echo ($version ?? 'no version') . "\n";

    echo processInput(null) . "\n";
    echo processInput(42) . "\n";
    echo processInput('hello') . "\n";
}
