<?php

declare(strict_types=1);

/**
 * @template T
 */
interface TypeInterface
{
    /**
     * @return T
     */
    public function coerce(mixed $value): mixed;
}

/**
 * @template T
 *
 * @implements TypeInterface<T>
 */
class Box implements TypeInterface
{
    /**
     * @return T
     */
    public function coerce(mixed $value): mixed
    {
        /** @var T */
        return $value;
    }

    /**
     * @return T
     */
    public function __invoke(mixed $value): mixed
    {
        /** @var T */
        return $value;
    }
}

/**
 * @extends Box<int>
 */
final class IntBox extends Box
{
}

/**
 * @return Box<string>
 */
function string_box(): Box
{
    return new Box();
}

/**
 * @return TypeInterface<non-empty-string>
 */
function non_empty_string_type(): TypeInterface
{
    return new Box();
}

function takes_string(string $_s): void
{
}

/**
 * @param non-empty-string $_s
 */
function takes_non_empty_string(string $_s): void
{
}

function takes_int(int $_i): void
{
}

/**
 * @template T
 *
 * @param Closure(mixed): T $closure
 *
 * @return T
 */
function call_closure(Closure $closure): mixed
{
    return $closure('a');
}

function generic_receivers(IntBox $intBox): void
{
    $viaArray = Closure::fromCallable([string_box(), 'coerce']);
    takes_string($viaArray('a'));

    $viaFirstClass = string_box()->coerce(...);
    takes_string($viaFirstClass('a'));

    $viaInterface = Closure::fromCallable([non_empty_string_type(), 'coerce']);
    takes_non_empty_string($viaInterface('a'));
    takes_non_empty_string(call_closure($viaInterface));

    $viaInvokable = Closure::fromCallable(string_box());
    takes_string($viaInvokable('a'));

    $viaInheritedMethod = Closure::fromCallable([$intBox, 'coerce']);
    takes_int($viaInheritedMethod('a'));

    $viaInheritedInvokable = Closure::fromCallable($intBox);
    takes_int($viaInheritedInvokable('a'));
}
