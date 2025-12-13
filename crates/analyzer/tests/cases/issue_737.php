<?php

declare(strict_types=1);

function create_array(): array
{
    return [1, '1', 'a', 3];
}

$array = create_array();

if ([] === $array) {
    die('a');
}

if (array_is_list($array)) {
    echo 'its a list!';
}

/** @param array{foo: string, bar: int} $a */
function string_keys_vs_list(array $a): void
{
    // @mago-expect analysis:impossible-type-comparison
    // @mago-expect analysis:impossible-condition
    if (array_is_list($a)) {
        echo 'never';
    }
}

/** @param array{0: string, 1: int} $a */
function int_keys_vs_list(array $a): void
{
    // @mago-expect analysis:redundant-condition
    // @mago-expect analysis:redundant-type-comparison
    if (array_is_list($a)) {
        echo 'always true - this is already a list shape';
    }
}

/** @param array<string, mixed> $a */
function string_key_type_vs_list(array $a): void
{
    // @mago-expect analysis:impossible-type-comparison
    // @mago-expect analysis:impossible-condition
    if (array_is_list($a)) {
        echo 'never';
    }
}

/** @param array<int, mixed> $a */
function int_key_type_vs_list(array $a): void
{
    if (array_is_list($a)) {
        echo 'possible';
    }
}

/** @param array<array-key, mixed> $a */
function array_key_type_vs_list(array $a): void
{
    if (array_is_list($a)) {
        echo 'possible';
    }
}

/** @param non-empty-array<string, mixed> $a */
function non_empty_string_key_vs_list(array $a): void
{
    // @mago-expect analysis:impossible-type-comparison
    // @mago-expect analysis:impossible-condition
    if (array_is_list($a)) {
        echo 'never';
    }
}

/** @param non-empty-array<int, mixed> $a */
function non_empty_int_key_vs_list(array $a): void
{
    if (array_is_list($a)) {
        echo 'possible';
    }
}

/** @param array{0: string, foo: int} $a */
function mixed_keys_vs_list(array $a): void
{
    // @mago-expect analysis:impossible-type-comparison
    // @mago-expect analysis:impossible-condition
    if (array_is_list($a)) {
        echo 'never';
    }
}

/** @param array{} $a */
function empty_shape_vs_list(array $a): void
{
    // @mago-expect analysis:redundant-condition
    // @mago-expect analysis:redundant-type-comparison
    if (array_is_list($a)) {
        echo 'always true - empty array is a list';
    }
}

/** @param list<mixed> $a */
function list_vs_list(array $a): void
{
    // @mago-expect analysis:redundant-condition
    // @mago-expect analysis:redundant-type-comparison
    if (array_is_list($a)) {
        echo 'always true';
    }
}

/** @param non-empty-list<mixed> $a */
function non_empty_list_vs_list(array $a): void
{
    // @mago-expect analysis:redundant-condition
    // @mago-expect analysis:redundant-type-comparison
    if (array_is_list($a)) {
        echo 'always true';
    }
}

/** @param list{string, int, bool} $a */
function list_with_known_elements(array $a): void
{
    // @mago-expect analysis:redundant-condition
    // @mago-expect analysis:redundant-type-comparison
    if (array_is_list($a)) {
        echo 'always true';
    }
}

/** @param array{foo: string, ...} $a */
function unsealed_string_keys(array $a): void
{
    // @mago-expect analysis:impossible-type-comparison
    // @mago-expect analysis:impossible-condition
    if (array_is_list($a)) {
        echo 'never';
    }
}

/** @param array{0: string, ...} $a */
function unsealed_int_keys(array $a): void
{
    if (array_is_list($a)) {
        echo 'possible';
    }
}
