<?php

declare(strict_types=1);

namespace Issue2409;

/** @return array{file: string, line: int} */
function caller(int $_off = 0): array
{
    $a = [];
    $a['file'] = 'unknown';
    $a['line'] = 0;

    return $a;
}

function take_string(?string $_str): void {}

function fatal_internal_error(): never
{
    $c = caller();
    foreach ($c as $_x => $_y) {
    }

    take_string($c['file']);
    exit(0);
}

/**
 * @param array{file: string, line: int} $record
 * @return array{file: string, line: int}
 */
function readOnlyShape(array $record): array
{
    foreach ($record as $key => $value) {
        echo $key, ': ', $value;
    }

    return $record;
}

/**
 * @param array{string, int, bool} $tuple
 * @return array{string, int, bool}
 */
function readOnlyTuple(array $tuple): array
{
    foreach ($tuple as $_key => $_value) {
    }

    return $tuple;
}

/**
 * @param array{file: string, line?: int} $record
 * @return array{file: string, line?: int}
 */
function optionalField(array $record): array
{
    foreach ($record as $_key => $_value) {
    }

    return $record;
}

/**
 * @param array{file: string, line: int} $record
 * @return array{file: string, line: int}
 */
function unrelatedAssignment(array $record): array
{
    $count = 0;
    foreach ($record as $_key => $_value) {
        $count++;
    }

    echo $count;

    return $record;
}

/**
 * @param array{file: string, line: int} $record
 * @return array{file: string, line: int}
 */
function valueOnly(array $record): array
{
    foreach ($record as $_value) {
    }

    return $record;
}

/**
 * @param array{file: string, line: int} $record
 * @return array{file: string, line: int}
 */
function keysOnly(array $record): array
{
    foreach (array_keys($record) as $_key) {
    }

    return $record;
}

/**
 * @param array<string, int|string> $values
 * @return array<string, string>
 * @throws \InvalidArgumentException
 */
function narrowValues(array $values): array
{
    foreach ($values as $_key => $value) {
        if (!is_string($value)) {
            throw new \InvalidArgumentException();
        }
    }

    return $values;
}

/**
 * @param array{file: string, line: int} $record
 * @return array{file: int, line: int}
 */
function convertAllValues(array $record): array
{
    foreach ($record as $key => $value) {
        if (is_string($value)) {
            $record[$key] = strlen($value);
        }
    }

    return $record;
}
