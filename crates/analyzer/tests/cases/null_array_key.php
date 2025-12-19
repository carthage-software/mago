<?php

function getStringOrNull(): ?string {
    return null;
}

/**
 * @return array<string, string>
 */
function testNullableKeyWarning(): array {
    $array = [];
    $k = getStringOrNull();
    $array[$k] = 'hello'; // @mago-expect analysis:possibly-null-array-index
    return $array;
}

/**
 * @return non-empty-array<string, string>
 */
function testNullableKeyType(): array {
    $array = [];
    $k = getStringOrNull();
    $array[$k] = 'hello'; // @mago-expect analysis:possibly-null-array-index
    return $array;
}

/**
 * @return array{'': string, ...} - array with explicit null key -> empty string key
 */
function testExplicitNullKey(): array {
    $array = [];
    $array[null] = 'value'; // @mago-expect analysis:null-array-index
    return $array;
}

/**
 * @param array<string, mixed> $arr
 */
function testNullKeyAccess(array $arr): mixed {
    $k = getStringOrNull();
    return $arr[$k]; // @mago-expect analysis:possibly-null-array-index
}
