<?php

declare(strict_types=1);

/**
 * @param positive-int $length
 */
function expand(string $seed, int $length): string
{
    $output = '';
    while (strlen($output) < $length) {
        $output .= hash('sha256', $seed, true);
    }

    return substr($output, 0, $length);
}

/**
 * @param int<0, max> $length
 *
 * @return ($length is 0 ? '' : non-empty-string)
 */
function random_text(int $length, string $alphabet): string
{
    $text = '';
    while ($length > 0) {
        $text .= $alphabet[0];
        --$length;
    }

    return $text;
}

/**
 * @template-covariant T
 */
final class Maybe
{
    /**
     * @param T $value
     */
    public function __construct(
        public readonly mixed $value,
    ) {}

    /**
     * @param Maybe<T> $other
     */
    public function equals(mixed $other): bool
    {
        return $this->value === $other->value;
    }
}

interface Scan
{
}

interface Hook
{
}

/**
 * @template T of Scan|Hook
 */
final class Registered
{
    /**
     * @param T $callback
     */
    public function __construct(
        public readonly object $callback,
    ) {}
}

/**
 * @template T of Scan|Hook
 *
 * @param list<T> $callbacks
 * @param list<Registered<T>> $all
 *
 * @param-out list<Registered<T>> $all
 */
function register(array $callbacks, array &$all): void
{
    foreach ($callbacks as $callback) {
        if ($callback instanceof Hook) {
            echo 'hook';
        }

        $all[] = new Registered($callback);
    }
}
