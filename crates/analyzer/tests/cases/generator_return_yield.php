<?php declare(strict_types=1);

/**
 * @template T
 */
final readonly class Identity
{
    public function __construct(
        /** @var T */
        public mixed $value,
    ) {}
}

/**
 * @template T
 * @param Identity<T> $identity
 * @return Generator<mixed, Identity<T>, T, T>
 */
function as_gen_return_yield(Identity $identity): Generator
{
    return yield $identity;
}

/**
 * @template T
 * @param Identity<T> $identity
 * @return Generator<mixed, Identity<T>, T, T>
 */
function as_gen_assign_then_return(Identity $identity): Generator
{
    $value = yield $identity;
    return $value;
}
