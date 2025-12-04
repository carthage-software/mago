<?php

class SomeClass
{
}

function create_some_instance(): SomeClass
{
    return new SomeClass();
}

/**
 * @psalm-assert !null $value
 */
function assert_not_null(mixed $value): void
{
    if ($value === null) {
        throw new InvalidArgumentException('Value cannot be null.');
    }
}

function process_value(null|int $value): int
{
    assert_not_null($value);
    return $value * 2;
}

function other(): void
{
    $instance = create_some_instance();
    assert_not_null($instance); // @mago-expect analysis:redundant-type-comparison
}
