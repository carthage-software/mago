<?php

declare(strict_types=1);

function decode_default(string $input): int
{
    return strlen(base64_decode($input));
}

function decode_lenient(string $input): int
{
    return strlen(base64_decode($input, false));
}

function decode_named(string $input): int
{
    return strlen(base64_decode(strict: false, string: $input));
}

function decode_strict(string $input): int
{
    // @mago-expect analysis:possibly-false-argument
    return strlen(base64_decode($input, true));
}

function decode_unknown(string $input, bool $strict): int
{
    // @mago-expect analysis:possibly-false-argument
    return strlen(base64_decode($input, $strict));
}
