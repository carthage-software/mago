<?php

class A
{
}

class B
{
}

class C
{
}

function say(string $message): int
{
    echo $message . "\n";

    return 2;
}

function factorial(int $number): int
{
    if ($number <= 1) {
        return 1;
    }

    return (int) ($number * factorial($number - 1));
}

function showObject(A|B|C $object): int
{
    return say('Object is of type ' . $object::class);
}

function showFactorial(int $number): int
{
    return say("Factorial of {$number} is " . factorial($number));
}

/**
 * @return array{
 *   'foo': null|string,
 *   'bar': null|int,
 *   'baz': bool,
 *   'qux': A|B|C|null,
 * }
 */
function example(): array
{
    return [
        'foo' => null,
        'bar' => 42,
        'baz' => true,
        'qux' => null,
    ];
}

/**
 * @param array{foo: null|string, bar: null|int, baz: bool, qux: A|B|C|null} $example
 * @return array{foo: int, bar: int, baz: bool, qux: int|null}
 */
function test(array $example): array
{
    return [
        'foo' => $example['foo'] !== null ? say($example['foo']) : 0,
        'bar' => $example['bar'] !== null ? showFactorial($example['bar']) : 0,
        'baz' => $example['baz'] === true ? false : true,
        'qux' => $example['qux'] !== null ? showObject($example['qux']) : null,
    ];
}

/**
 * @template K as array-key
 * @template V
 * @template U
 *
 * @param callable(V): U $callback
 *
 * @param array<K, V> $array
 * @return array<K, U>
 */
function my_array_map(callable $callback, array $array): array
{
    $result = [];
    foreach ($array as $key => $value) {
        $result[$key] = $callback($value);
    }
    return $result;
}

$result = test(example());

showFactorial($result['foo']);
showFactorial($result['bar']);

if ($result['qux'] !== null) {
    showFactorial($result['qux']);
}

/** @var list<array{0: null|A|B|C, 1: null|A|B|C, 2: bool}> $types */
$types = [
    [new A(), new B(), true],
    [new B(), new C(), false],
    [new C(), null, true],
    [null, null, false],
    [null, new A(), true],
    [new A(), new A(), true],
];

$result = my_array_map(
    /**
     * @param array{
     *   0: null|A|B|C,
     *   1: null|A|B|C,
     *   2: bool
     * } $types
     *
     * @return array{
     *   0: null|int,
     *   1: null|int,
     *   2: bool
     * }
     */
    static fn(array $types): array => [
        $types[0] === null ? null : showObject($types[0]),
        $types[1] === null ? null : showObject($types[1]),
        $types[2],
    ],
    $types,
);
