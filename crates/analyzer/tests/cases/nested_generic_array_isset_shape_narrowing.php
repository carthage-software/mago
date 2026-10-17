<?php

declare(strict_types=1);

/**
 * @param array{a: array<string, int>} $array
 * @return array{a: array{b: int, ...}}
 * @throws \LogicException
 */
function test_nested_shape_narrowing(array $array): array
{
    if (!isset($array["a"]["b"])) {
        throw new \LogicException();
    }

    return $array;
}

/**
 * @param array<string, array<string, string>> $data
 */
function test_nested_generic_narrowing(array $data): void
{
    if (!isset($data['foo']['bar'])) {
        return;
    }

    accept_nested_shape($data);
}

/**
 * @param array{'foo': array{'bar': string, ...}, ...} $_
 */
function accept_nested_shape(array $_): void {}
