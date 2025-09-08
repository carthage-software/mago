<?php

declare(strict_types=1);

class Pair
{
    public function __construct(
        public string $key,
        public int $value,
    ) {}
}

/**
 * @param list<Pair> $array
 * @mago-expect analysis:type-confirmation
 */
function value_key(array $array): array {
    $v = array_column($array, 'value', 'key');
    Mago\confirm($v, 'array<string, int>');
    return $v;
}

/**
 * @param list<Pair> $array
 * @mago-expect analysis:type-confirmation
 */
function no_value_key(array $array): array {
    $v = array_column($array, null, 'key');
    Mago\confirm($v, 'array<string, Pair>');
    return $v;
}

/**
 * @param list<Pair> $array
 * @mago-expect analysis:type-confirmation
 */
function no_value_no_key(array $array): array {
    $v = array_column($array, null);
    Mago\confirm($v, 'list<Pair>');
    return $v;
}

/**
 * @param list<Pair> $array
 * @mago-expect analysis:type-confirmation
 */
function value_no_key(array $array): array {
    $v = array_column($array, 'value', null);
    Mago\confirm($v, 'list<int>');
    return $v;
}

$v = [new Pair('a', 1)];
$column = array_column($v, 'value');

/** @mago-expect analysis:type-confirmation */
Mago\confirm($column, 'list<int>');
