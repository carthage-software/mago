<?php

declare(strict_types=1);

function processValue(int|string|null $value): string
{
    if ($value === null) {
        return 'No value';
    }

    if (is_string($value)) {
        return 'String: ' . $value;
    }

    return 'Number: ' . $value;
}

function validateInput(int|string $input): int
{
    if (is_string($input)) {
        if (!ctype_digit($input)) {
            return 0;
        }
        $input = (int) $input;
    }

    if ($input < 0) {
        return 0;
    }

    if ($input > 1000) {
        return 1000;
    }

    return $input;
}

function processData(array|object|null $data): string
{
    if ($data === null) {
        return 'No data';
    }

    if (is_array($data)) {
        return 'Array with ' . count($data) . ' items';
    }

    return 'Object of type ' . get_class($data);
}

function test(): void
{
    echo processValue(null) . "\n";
    echo processValue('hello') . "\n";
    echo processValue(42) . "\n";

    echo validateInput(42) . "\n";
    echo validateInput('100') . "\n";

    echo processData(null) . "\n";
    echo processData(['a', 'b']) . "\n";
    echo processData(new stdClass()) . "\n";
}
