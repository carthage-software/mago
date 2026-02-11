<?php declare(strict_types=1);

function test_filter_validate_int(mixed $value): false|int
{
    return filter_var($value, FILTER_VALIDATE_INT);
}

function test_filter_validate_float(mixed $value): false|float
{
    return filter_var($value, FILTER_VALIDATE_FLOAT);
}

function test_filter_validate_bool(mixed $value): bool
{
    return filter_var($value, FILTER_VALIDATE_BOOLEAN);
}

function test_filter_validate_url(mixed $value): false|string
{
    return filter_var($value, FILTER_VALIDATE_URL);
}

function test_filter_validate_email(mixed $value): false|string
{
    return filter_var($value, FILTER_VALIDATE_EMAIL);
}

function test_filter_validate_ip(mixed $value): false|string
{
    return filter_var($value, FILTER_VALIDATE_IP);
}

function test_filter_validate_mac(mixed $value): false|string
{
    return filter_var($value, FILTER_VALIDATE_MAC);
}

function test_filter_validate_domain(mixed $value): false|string
{
    return filter_var($value, FILTER_VALIDATE_DOMAIN);
}

function test_filter_validate_regexp(mixed $value): false|string
{
    return filter_var($value, FILTER_VALIDATE_REGEXP, ['options' => ['regexp' => '/^[a-z]+$/']]);
}

function test_filter_sanitize_email(mixed $value): string
{
    return filter_var($value, FILTER_SANITIZE_EMAIL);
}

function test_filter_sanitize_url(mixed $value): string
{
    return filter_var($value, FILTER_SANITIZE_URL);
}

function test_filter_sanitize_number_int(mixed $value): string
{
    return filter_var($value, FILTER_SANITIZE_NUMBER_INT);
}

function test_filter_sanitize_add_slashes(mixed $value): string
{
    return filter_var($value, FILTER_SANITIZE_ADD_SLASHES);
}

function test_filter_default(mixed $value): string
{
    return filter_var($value, FILTER_DEFAULT);
}

function test_filter_null_on_failure(mixed $value): ?int
{
    return filter_var($value, FILTER_VALIDATE_INT, FILTER_NULL_ON_FAILURE);
}

function test_filter_validate_bool_null_on_failure(mixed $value): ?bool
{
    return filter_var($value, FILTER_VALIDATE_BOOLEAN, FILTER_NULL_ON_FAILURE);
}

function test_filter_input_validate_int(): false|int
{
    return filter_input(INPUT_GET, 'id', FILTER_VALIDATE_INT);
}

function test_filter_input_sanitize(): string
{
    return filter_input(INPUT_GET, 'search', FILTER_SANITIZE_SPECIAL_CHARS);
}

function test_filter_input_null_on_failure(): ?string
{
    return filter_input(INPUT_GET, 'search', FILTER_VALIDATE_EMAIL, FILTER_NULL_ON_FAILURE);
}
