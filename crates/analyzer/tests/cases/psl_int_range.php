<?php

declare(strict_types=1);

namespace Psl\Type {
    /**
     * @template-covariant T
     */
    interface TypeInterface
    {
        /**
         * @psalm-assert-if-true T $value
         */
        public function matches(mixed $value): bool;

        /**
         * @return T
         */
        public function coerce(mixed $value): mixed;

        /**
         * @return T
         *
         * @psalm-assert T $value
         */
        public function assert(mixed $value): mixed;
    }

    /**
     * @return TypeInterface<int>
     */
    function int_range(int $min, int $max): TypeInterface
    {
        return int_range($min, $max);
    }
}

namespace App {
    use Psl\Type;

    function get_mixed(): mixed
    {
        return get_mixed();
    }

    function get_unknown_int(): int
    {
        return get_unknown_int();
    }

    /**
     * @param int<1, 5> $a
     */
    function takes_int_1_5(int $a): void
    {
        takes_int_1_5($a);
    }

    /**
     * @param int<0, 100> $a
     */
    function takes_int_0_100(int $a): void
    {
        takes_int_0_100($a);
    }

    /**
     * @param int<-10, 10> $a
     */
    function takes_int_neg10_10(int $a): void
    {
        takes_int_neg10_10($a);
    }

    takes_int_1_5(Type\int_range(1, 5)->assert(get_mixed()));

    takes_int_1_5(Type\int_range(2, 4)->assert(get_mixed()));

    takes_int_0_100(Type\int_range(0, 100)->assert(get_mixed()));

    takes_int_0_100(Type\int_range(10, 50)->assert(get_mixed()));

    takes_int_neg10_10(Type\int_range(-10, 10)->assert(get_mixed()));

    takes_int_neg10_10(Type\int_range(-5, 5)->assert(get_mixed()));

    /**
     * @param int<0, 255> $byte
     */
    function takes_byte(int $byte): void
    {
        takes_byte($byte);
    }

    takes_byte(Type\int_range(0, 255)->coerce(get_mixed()));

    $type = Type\int_range(1, 100);
    /** @var mixed $value */
    $value = null;
    if ($type->matches($value)) {
        takes_int_0_100($value);
    }
}
