<?php

declare(strict_types=1);

/** @psalm-consistent-constructor */
abstract class IsAMessage {}

/**
 * @template T of object
 * @param class-string<T> $className
 * @return T
 * @throws LogicException
 */
function is_a_template_make(string $className): object
{
    if (!is_a($className, IsAMessage::class, true)) {
        throw new LogicException();
    }

    return new $className();
}

/**
 * @template T of object
 * @param class-string<T> $className
 * @return T
 * @throws LogicException
 */
function is_a_template_build(string $className): object
{
    if (is_subclass_of($className, IsAMessage::class)) {
        return new $className();
    }

    throw new LogicException();
}

interface IsAHasName {}
interface IsAHasId {}
final class IsABoth implements IsAHasName, IsAHasId {}

/**
 * @template T of IsAHasName
 * @param class-string<T> $className
 * @return class-string<T>
 * @throws LogicException
 */
function is_a_template_pick(string $className): string
{
    if (!is_subclass_of($className, IsAHasId::class)) {
        throw new LogicException();
    }

    return $className;
}

