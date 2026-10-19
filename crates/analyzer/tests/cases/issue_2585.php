<?php

declare(strict_types=1);

namespace Issue2585;

enum Database: string
{
    case TEST = 'test';
    case TEST2 = 'test3';
}

/** @param truthy-string $_ */
function takes_truthy_string(string $_): void {}

/** @var array{test: string, id: int, date: \DateTime}|null $a */
$a = [
    'test' => '',
    'id' => 1,
    'date' => new \DateTime(),
];

if (empty($a[Database::TEST->value])) {
    die();
}

takes_truthy_string($a['test']);
takes_truthy_string($a[Database::TEST->value]);

/** @param array{test?: string, id: int}|null $a */
function optional_key(?array $a): void
{
    if (!empty($a[Database::TEST->value])) {
        takes_truthy_string($a['test']);
        takes_truthy_string($a[Database::TEST->value]);
    }
}

/** @param array{test?: array{test?: string, id: int}, id: int}|null $a */
function nested_key(?array $a): void
{
    if (empty($a[Database::TEST->value][Database::TEST->value])) {
        return;
    }

    takes_truthy_string($a['test']['test']);
    takes_truthy_string($a[Database::TEST->value][Database::TEST->value]);
}

enum Index: int
{
    case FIRST = 0;
}

/** @param array{0?: string, id: int}|null $a */
function integer_key(?array $a): void
{
    if (empty($a[Index::FIRST->value])) {
        return;
    }

    takes_truthy_string($a[0]);
    takes_truthy_string($a[Index::FIRST->value]);
}

/** @param array{test?: string, id: int}|null $a */
function variable_key(?array $a): void
{
    $key = Database::TEST->value;

    if (empty($a[$key])) {
        return;
    }

    takes_truthy_string($a['test']);
    takes_truthy_string($a[$key]);
}

/** @param array{test: string, test3: string, id: int} $a */
function unknown_key(array $a, Database $key): void
{
    if (empty($a[$key->value])) {
        return;
    }

    // @mago-expect analysis:possibly-invalid-argument
    takes_truthy_string($a['test']);
    // @mago-expect analysis:possibly-invalid-argument
    takes_truthy_string($a['test3']);
}
