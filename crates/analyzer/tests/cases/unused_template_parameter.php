<?php

/**
 * @template T
 *
 * @mago-expect analysis:unused-template-parameter
 */
final class UnusedTemplate
{
    // T is never used in properties, methods, or inherited types
}

/**
 * @template T
 */
final class UsedInProperty
{
    /** @var T */
    public mixed $value;
}

/**
 * @template T
 */
final class UsedInMethodParam
{
    /** @param T $value */
    public function set(mixed $value): void
    {
    }
}

/**
 * @template T
 */
final class UsedInReturn
{
    /** @return T */
    public function get(): mixed
    {
        return null;
    }
}

/**
 * @template _T
 */
final class IntentionallyUnused
{
}

/**
 * @template T
 * @template U
 *
 * @mago-expect analysis:unused-template-parameter
 */
final class PartiallyUsed
{
    /** @var T */
    public mixed $value;
}

/**
 * @template T
 */
interface UsedInExtendsInterface
{
    /** @return null|T */
    public function get(): mixed;
}

/**
 * @template T
 *
 * @implements UsedInExtendsInterface<T>
 */
final class UsedInImplements implements UsedInExtendsInterface
{
    public function get(): mixed
    {
        return null;
    }
}

/**
 * @template T
 */
abstract class UsedInExtendsBase
{
    /** @var T */
    public mixed $value;
}

/**
 * @template T
 *
 * @extends UsedInExtendsBase<T>
 */
final class UsedInExtendsChild extends UsedInExtendsBase
{
}

/**
 * @template T
 *
 * @mago-expect analysis:unused-template-parameter
 */
abstract class UnusedInAbstract
{
    // Abstract classes should still be checked
}

/**
 * @template T
 *
 * @mago-expect analysis:unused-template-parameter
 */
trait UnusedInTrait
{
    // Traits should still be checked
}

/**
 * @template T
 */
trait UsedInTraitProperty
{
    /** @var T */
    public mixed $value;
}
