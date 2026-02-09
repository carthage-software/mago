<?php

declare(strict_types=1);

/**
 * @psalm-assert non-empty-string $value
 *
 * @throws InvalidArgumentException
 */
function assert_non_empty_string(mixed $value): void
{
    if (!is_string($value) || $value === '') {
        throw new InvalidArgumentException('Expected non-empty string');
    }
}

/**
 * @psalm-assert !null $value
 *
 * @throws InvalidArgumentException
 */
function assert_non_null_831(mixed $value): void
{
    if ($value === null) {
        throw new InvalidArgumentException('Value cannot be null');
    }
}

/**
 * @return non-empty-string
 */
function get_field(): string
{
    return 'field_name';
}

class SomeEntity
{
}

function create_entity(): SomeEntity
{
    return new SomeEntity();
}

/**
 * @throws InvalidArgumentException
 */
function test_unconditional_assert_not_redundant(): void
{
    $field = get_field();
    // $field is already non-empty-string, but assert_non_empty_string() does runtime validation.
    // So this is NOT redundant.
    assert_non_empty_string($field);

    $entity = create_entity();
    // $entity is already SomeEntity(not null), but runtime guard is intentional.
    assert_non_null_831($entity);
}
