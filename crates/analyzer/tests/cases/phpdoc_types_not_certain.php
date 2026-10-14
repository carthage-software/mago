<?php

declare(strict_types=1);

namespace App;

final class Result
{
    /**
     * @param int $code
     */
    public function __construct($code)
    {
        if (!is_int($code)) {
            throw new \InvalidArgumentException('code must be an int, got ' . get_debug_type($code));
        }
    }
}

/** @template T of object */
interface Factory
{
    /** @param T $input */
    public function create(object $input): string;
}

final class Order
{
}

/** @implements Factory<Order> */
final class OrderFactory implements Factory
{
    public function create(object $input): string
    {
        if (!$input instanceof Order) {
            throw new \InvalidArgumentException('Expected ' . Order::class . ', got ' . $input::class);
        }

        return 'order';
    }
}

final class Validator
{
    /** @param \Countable $value */
    public function validate($value): void
    {
        if (!$value instanceof \Countable) {
            throw new \UnexpectedValueException(get_debug_type($value));
        }
    }
}

final class Filter
{
    public function __construct(
        public string $name,
    ) {}
}

interface FilterBuilder
{
    /** @return Filter */
    public function build(string $name): ?Filter;
}

final class Service
{
    public function __construct(
        private FilterBuilder $builder,
    ) {}

    public function names(string $name): string
    {
        $filter = $this->builder->build($name);
        if ($filter && $filter->name !== '') {
            return $filter->name;
        }

        return '';
    }
}

final class BitMask
{
    private int $length;

    /** @param int $length */
    public function __construct($length)
    {
        if (!is_numeric($length)) {
            throw new \InvalidArgumentException('length must be numeric');
        }
        $this->length = (int) $length;
    }

    public function length(): int
    {
        return $this->length;
    }
}

function test(): void
{
    new Result('7'); // @mago-expect analysis:invalid-argument
}
