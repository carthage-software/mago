<?php

declare(strict_types=1);

class TypeChecker {
    /**
     * @psalm-assert-if-true string $value
     */
    public static function isString(mixed $value): bool {
        return is_string($value);
    }

    /**
     * @psalm-assert-if-true int $value
     */
    public static function isInt(mixed $value): bool {
        return is_int($value);
    }

    /**
     * @psalm-assert-if-true bool $value
     */
    public static function isBool(mixed $value): bool {
        return is_bool($value);
    }
}

/**
 * @param array<int, string|int> $values
 * @return array<int, string>
 */
function filterStringsUsingFunc(array $values): array
{
    return array_filter($values, is_string(...));
}

/**
 * @param array<int, string|int> $values
 * @return array<int, int>
 */
function filterIntsUsingFunc(array $values): array
{
    return array_filter($values, is_int(...));
}

/**
 * @param array<int, string|int|bool> $values
 * @return array<int, bool>
 */
function filterBoolsUsingFunc(array $values): array
{
    return array_filter($values, is_bool(...));
}

/**
 * @param array<int, string|int> $values
 * @return array<int, string>
 */
function filterStringsUsingStaticMethod(array $values): array
{
    return array_filter($values, TypeChecker::isString(...));
}

/**
 * @param array<int, string|int> $values
 * @return array<int, int>
 */
function filterIntsUsingStaticMethod(array $values): array
{
    return array_filter($values, TypeChecker::isInt(...));
}

/**
 * @param array<int, string|int|bool> $values
 * @return array<int, bool>
 */
function filterBoolsUsingStaticMethod(array $values): array
{
    return array_filter($values, TypeChecker::isBool(...));
}

/**
 * @param array<int, string|int> $values
 * @return array<int, string>
 */
function filterStringsUsingFuncString(array $values): array
{
    return array_filter($values, 'is_string');
}

/**
 * @param array<int, string|int> $values
 * @return array<int, int>
 */
function filterIntsUsingFuncString(array $values): array
{
    return array_filter($values, 'is_int');
}

/**
 * @param array<int, string|int|bool> $values
 * @return array<int, bool>
 */
function filterBoolsUsingFuncString(array $values): array
{
    return array_filter($values, 'is_bool');
}

/**
 * @param array<int, string|int> $values
 * @return array<int, string>
 */
function filterStringsUsingStaticMethodString(array $values): array
{
    return array_filter($values, 'TypeChecker::isString');
}

/**
 * @param array<int, string|int> $values
 * @return array<int, int>
 */
function filterIntsUsingStaticMethodString(array $values): array
{
    return array_filter($values, 'TypeChecker::isInt');
}

/**
 * @param array<int, string|int|bool> $values
 * @return array<int, bool>
 */
function filterBoolsUsingStaticMethodString(array $values): array
{
    return array_filter($values, 'TypeChecker::isBool');
}
