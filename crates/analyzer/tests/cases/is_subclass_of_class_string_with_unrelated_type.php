<?php

declare(strict_types=1);

interface Customer
{
    public static function prefix(): string;
}

interface HasA
{
    public static function letter(): string;
}

abstract class Model
{
    public static function table(): string
    {
        return 'models';
    }
}

/**
 * @param class-string<Customer> $class
 *
 * @throws LogicException
 */
function negated_check(string $class): string
{
    if (!is_subclass_of($class, HasA::class)) {
        throw new LogicException('unsupported');
    }

    // @mago-expect analysis:possibly-static-access-on-interface
    return $class::prefix();
}

/** @param class-string<Customer> $class */
function positive_check(string $class): string
{
    if (is_subclass_of($class, HasA::class)) {
        // @mago-expect analysis:possibly-static-access-on-interface
        return $class::letter();
    }

    return '';
}

/** @param class-string<Customer> $class */
function class_check(string $class): string
{
    if (is_a($class, Model::class, true)) {
        return $class::table();
    }

    return '';
}
