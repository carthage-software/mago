<?php

/**
 * @template T
 */
interface Type
{
    /** @param T $value */
    public function isValid(mixed $value): bool;
}

/**
 * @template T
 */
interface Entry
{
    /** @return T */
    public function value(): mixed;
}

/**
 * @template TKey of array-key
 * @template TValue
 *
 * @param ?array<array-key, mixed> $value
 * @param Type<array<TKey, TValue>> $mapType
 *
 * @return Entry<null|array<TKey, TValue>>
 */
function map_entry(string $name, null|array $value, Type $mapType): Entry
{
    return map_entry($name, $value, $mapType);
}

/**
 * @template TKey of array-key
 * @template TValue
 *
 * @param Type<TKey> $keyType
 * @param Type<TValue> $valueType
 *
 * @return Type<array<TKey, TValue>>
 */
function type_map(Type $keyType, Type $valueType): Type
{
    return type_map($keyType, $valueType);
}

/**
 * @return Type<string>
 */
function type_string(): Type
{
    return type_string();
}

/**
 * @param array<string, string> $map
 */
function use_string_map(array $map): void
{
    foreach ($map as $key => $value) {
        echo $key . ' => ' . $value . "\n";
    }
}

$v = map_entry('strings', ['one' => 'two'], type_map(type_string(), type_string()))->value();
if ($v !== null) {
    use_string_map($v);
}
