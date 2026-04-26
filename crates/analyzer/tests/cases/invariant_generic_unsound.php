<?php

/**
 * @template T
 */
final class Cell
{
    /**
     * @var T
     */
    public mixed $value;

    /**
     * @param T $value
     */
    public function __construct(mixed $value)
    {
        $this->value = $value;
    }

    /**
     * @param T $value
     */
    public function set(mixed $value): void
    {
        $this->value = $value;
    }

    /**
     * @return T
     */
    public function get(): mixed
    {
        return $this->value;
    }
}

/**
 * @param Cell<scalar> $cell
 */
function store_string_into_scalar_cell(Cell $cell): void
{
    $cell->set('hello world');
}

/**
 * @param Cell<int> $cell
 */
function increment_int_cell(Cell $cell): int
{
    return $cell->get() + 1;
}

/**
 * @mago-expect analysis:invalid-argument
 */
function exploit(): int
{
    $cell = new Cell(42);
    store_string_into_scalar_cell($cell);
    return increment_int_cell($cell);
}
