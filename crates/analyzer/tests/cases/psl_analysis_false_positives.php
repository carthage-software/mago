<?php

declare(strict_types=1);

/**
 * @template Tk of array-key
 * @template Tv
 *
 * @param iterable<Tk, Tv> $iterable
 *
 * @return array<Tk, Tv>
 */
function psl_to_array(iterable $iterable): array
{
    return [];
}

/**
 * @template Tk of array-key
 * @template Tv of scalar|null|resource|Stringable
 *
 * @param iterable<Tk, Tv> $first
 * @param iterable<Tk, Tv> $second
 * @param iterable<Tk, Tv> ...$rest
 *
 * @return array<Tk, Tv>
 */
function psl_diff(iterable $first, iterable $second, iterable ...$rest): array
{
    return array_diff(psl_to_array($first), psl_to_array($second), ...array_map(psl_to_array(...), $rest));
}

/**
 * @template Tk
 * @template Tv
 *
 * @param iterable<Tk, Tv> $iterable
 */
function psl_map(iterable $iterable): void
{
    if (is_array($iterable) && array_is_list($iterable)) {
    }
}

/** @suspends-fiber */
function psl_suspend(): void {}

function psl_resource_is_rechecked_after_suspend(mixed $resource): void
{
    if (!is_resource($resource)) {
        return;
    }

    psl_suspend();

    if (!is_resource($resource)) {
    }
}

enum PslBackedEnum: int
{
    case One = 1;
}

/**
 * @template T of BackedEnum
 */
final class PslBackedEnumValueType
{
    /** @psalm-assert-if-true value-of<T> $value */
    public function matches(mixed $value): bool
    {
        return true;
    }

    /** @return value-of<T> */
    public function coerce(mixed $value): string|int
    {
        $case = $value === null ? 1 : 'one';
        if ($this->matches($case)) {
            return $case;
        }

        exit();
    }
}

function psl_zero_iteration_while(string $bytes): void
{
    $i = 0;
    $length = strlen($bytes);
    while ($i < ($length - 1) && ord($bytes[$i]) === 0) {
        $i++;
    }

    if ($i > 0) {
    }
}

/** @param positive-int $value */
function psl_accepts_positive(int $value): void {}

/** @param int<0, max> $limit */
function psl_range_bound_does_not_narrow_the_else_branch(string $payload, int $limit): void
{
    $length = strlen($payload);
    if ($length > $limit) {
        psl_accepts_positive($length);

        return;
    }

    if ($length !== 5) {
    }
}

/** @param positive-int $size */
function psl_subtraction_respects_the_active_comparison(string $payload, int $size): void
{
    $length = strlen($payload);
    if ($length >= $size) {
        return;
    }

    $toRead = $size - $length;
    psl_accepts_positive($toRead);
}

/** @param positive-int $size */
function psl_subtraction_respects_an_or_guard(string $payload, int $size, bool $eof): void
{
    do {
        $length = strlen($payload);
        if ($length >= $size || $eof) {
            break;
        }

        $toRead = $size - $length;
        psl_accepts_positive($toRead);
    } while (true);
}

/**
 * @param positive-int $size
 * @param int<0, max> $replacement
 */
function psl_subtraction_forgets_relations_after_assignment(string $payload, int $size, int $replacement): void
{
    $length = strlen($payload);
    if ($length < $size) {
        $size = $replacement;
        $toRead = $size - $length;

        // @mago-expect analysis:possibly-invalid-argument
        psl_accepts_positive($toRead);
    }
}
