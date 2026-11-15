<?php

declare(strict_types=1);

namespace Psl\Iter {
    /**
     * @template T
     *
     * @param iterable<T> $iterable
     * @param T $value
     */
    function contains(iterable $iterable, mixed $value): bool
    {
        foreach ($iterable as $v) {
            if ($value === $v) {
                return true;
            }
        }

        return false;
    }
}

namespace PslContainsNarrowing {
    use function Psl\Iter\contains;

    /**
     * @psalm-assert false $condition
     */
    function assert_false(mixed $condition): void
    {
    }

    enum Code: string
    {
        case A = 'a';
        case B = 'b';
        case X = 'x';
    }

    /**
     * @param list<'a'|'b'|'x'> $values
     */
    function absent_value(array $values): void
    {
        assert_false(contains($values, Code::X->value));
    }

    /**
     * @param list<string> $a
     * @param list<string> $b
     */
    function different_iterables(array $a, array $b, string $needle): string
    {
        if (contains($a, $needle)) {
            return 'a';
        } elseif (contains($b, $needle)) {
            return 'b';
        }

        return 'none';
    }

    /**
     * @param 3 $_
     */
    function accept_three(int $_): void
    {
    }

    /**
     * @param 1|2|3 $needle
     */
    function required_values(int $needle): void
    {
        if (!contains([1, 2], $needle)) {
            accept_three($needle);
        }
    }

    /**
     * @param 'a'|'b' $_
     */
    function accept_letter(string $_): void
    {
    }

    /**
     * @param list<'a'|'b'> $values
     */
    function positive_membership(array $values, string $needle): void
    {
        if (contains($values, $needle)) {
            accept_letter($needle);
        }
    }
}
