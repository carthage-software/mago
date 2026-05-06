<?php

enum Color: string
{
    case Red = 'red';
    case Green = 'green';
    case Blue = 'blue';
}

/**
 * @template K of array-key
 * @template V
 * @template U
 *
 * @param array<K, V> $input
 * @param callable(V): U $callback
 *
 * @return (
 *   $input is list<V> ?
 *   ($input is non-empty-list ? non-empty-list<U> : list<U>) :
 *   ($input is non-empty-array ? non-empty-array<K, U> : array<K, U>)
 * )
 */
function map(array $input, callable $callback): array
{
    $result = [];
    foreach ($input as $k => $item) {
        $result[$k] = $callback($item);
    }

    return $result;
}

/**
 * @param non-empty-string $value
 */
function print_string(string $value): void
{
    echo $value . "\n";
}

/**
 * @param int<3, 15> $value
 */
function print_integer(int $value): void
{
    echo $value . "\n";
}

$entries = [
    ['a' => 1, 'b' => 2],
    ['a' => 3, 'b' => 4],
    ['a' => 5, 'b' => 6],
    ['a' => 7, 'b' => 8],
];

$result = map($entries, function (array $input): int {
    return $input['a'] + $input['b'];
});

foreach ($result as $item) {
    print_integer($item);
}

$entries = [
    ['color' => Color::Red, 'value' => 1, 'separator' => ' - '],
    ['color' => Color::Green, 'value' => 2, 'separator' => ' => '],
    ['color' => Color::Blue, 'value' => 3, 'separator' => ' @ '],
];

$result = map(
    $entries,
    // The type of `$input` is inferred as `array{color: Color, value: int, separator: string}`
    fn(array $input): string => $input['color']->value . $input['separator'] . $input['value'],
);

foreach ($result as $item) {
    print_string($item);
}
