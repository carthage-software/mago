<?php

class Item
{
    public function isValid(): bool
    {
        return true;
    }
}

class Converter
{
    public function convert(string $val): ?Item
    {
        return $val === '' ? null : new Item();
    }
}

/**
 * @throws RuntimeException
 */
function convertOrFail(Converter $converter, string $value): Item
{
    if (($value = $converter->convert($value)) === null || !$value->isValid()) {
        throw new RuntimeException();
    }

    return $value;
}

function convertNonExiting(Converter $converter, string $value): ?Item
{
    if (($value = $converter->convert($value)) === null || !$value->isValid()) {
        return $value;
    }

    return $value;
}

function convertNonExitingMustBeNullable(Converter $converter, string $value): Item
{
    if (($value = $converter->convert($value)) === null || !$value->isValid()) {
    }

    // @mago-expect analysis:nullable-return-statement,invalid-return-statement
    return $value;
}

function convertAnd(Converter $converter, string $value): ?Item
{
    if ($value !== '' && ($value = $converter->convert($value)) !== null) {
        return $value;
    }

    return null;
}

/**
 * @throws RuntimeException
 */
function convertNestedLogical(Converter $converter, string $value, bool $extra): Item
{
    if ((($value = $converter->convert($value)) === null || !$value->isValid()) || $extra) {
        throw new RuntimeException();
    }

    return $value;
}

/**
 * @throws RuntimeException
 */
function convertNestedIf(Converter $converter, string $value): Item
{
    if ($value !== '') {
        if (($value = $converter->convert($value)) === null || !$value->isValid()) {
            throw new RuntimeException();
        }

        return $value;
    }

    throw new RuntimeException();
}

/**
 * @throws RuntimeException
 */
function convertInvalidInput(Converter $converter, int $value): Item
{
    // @mago-expect analysis:invalid-argument
    if (($value = $converter->convert($value)) === null || !$value->isValid()) {
        throw new RuntimeException();
    }

    return $value;
}
