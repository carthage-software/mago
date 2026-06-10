<?php

declare(strict_types=1);

/**
 * @template T
 */
final class Box1966
{
    /**
     * @param T $contained
     */
    public function __construct(
        private readonly mixed $contained,
    ) {}

    /**
     * @param callable(T): void $cb
     */
    public function apply(callable $cb): void
    {
        $cb($this->contained);
    }
}

/**
 * @param Box1966<int|null> $box
 *
 * @mago-expect analysis:invalid-argument
 */
function rejects_non_nullable_callback(Box1966 $box): void
{
    $box->apply(static function (int $value): void {
        echo "You gave me {$value}";
    });
}

/**
 * @param Box1966<int|null> $box
 */
function accepts_nullable_callback(Box1966 $box): void
{
    $box->apply(static function (?int $value): void {
        echo "You gave me {$value}";
    });
}

/**
 * @param Box1966<int|float> $box
 */
function accepts_exact_union_callback(Box1966 $box): void
{
    $box->apply(static function (int|float $value): void {
        echo "You gave me {$value}";
    });
}

/**
 * @param Box1966<int|float> $box
 *
 * @mago-expect analysis:invalid-argument
 */
function rejects_narrower_union_callback(Box1966 $box): void
{
    $box->apply(static function (int $value): void {
        echo "You gave me {$value}";
    });
}
