<?php

declare(strict_types=1);

/**
 * @template T of object
 *
 * @param class-string<T> $expected
 *
 * @psalm-assert T $actual
 */
function assert_generic_instance_of(string $expected, mixed $actual): void
{
}

/**
 * @template T
 */
interface AssertedBox
{
    /**
     * @return T
     */
    public function get(): mixed;
}

/**
 * @template T
 *
 * @implements AssertedBox<T>
 */
final class AssertedListBox implements AssertedBox
{
    /**
     * @param T $value
     */
    public function __construct(
        private mixed $value,
    ) {}

    public function get(): mixed
    {
        return $this->value;
    }
}

/**
 * @param AssertedBox<int> $box
 */
function assertion_narrows_generic_to_descendant(AssertedBox $box): int
{
    assert_generic_instance_of(AssertedListBox::class, $box);

    return $box->get();
}
