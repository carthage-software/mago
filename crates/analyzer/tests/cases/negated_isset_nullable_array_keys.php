<?php

declare(strict_types=1);

/** @param array{value: null, label: string} $_ */
function acceptRequiredNull(array $_): void {}

/** @param array{value?: null, label: string} $_ */
function acceptOptionalNull(array $_): void {}

/** @param list{null, string} $_ */
function acceptNullTuple(array $_): void {}

/** @param list{string, 1?: null} $_ */
function acceptOptionalNullTuple(array $_): void {}

function acceptNull(null $_): void {}

function acceptString(string $_): void {}

/** @param array{value: ?string, label: string} $array */
function requiredNullableKey(array $array): ?string
{
    if (!isset($array['value'])) {
        acceptNull($array['value']);
        acceptRequiredNull($array);

        if (array_key_exists('value', $array)) {
            acceptNull($array['value']);
        }

        return $array['value'];
    }

    acceptString($array['value']);

    return $array['value'];
}

/** @param array{value?: ?string, label: string} $array */
function optionalNullableKey(array $array): ?string
{
    if (!isset($array['value'])) {
        acceptOptionalNull($array);

        if (array_key_exists('value', $array)) {
            acceptNull($array['value']);
            acceptRequiredNull($array);
        }

        // @mago-expect analysis:possibly-undefined-string-array-index
        return $array['value'];
    }

    acceptString($array['value']);

    return $array['value'];
}

/** @param array{value?: ?string, label: string} $array */
function absentNullableKey(array $array): void
{
    if (!array_key_exists('value', $array)) {
        // @mago-expect analysis:undefined-string-array-index
        acceptNull($array['value']);
    }
}

/** @param array{value?: string, label: string} $array */
function absentNonNullableKey(array $array): void
{
    if (!isset($array['value'])) {
        // @mago-expect analysis:undefined-string-array-index
        acceptNull($array['value']);
    }
}

/** @param list{?string, string} $array */
function requiredNullableTupleEntry(array $array): void
{
    if (!isset($array[0])) {
        acceptNull($array[0]);
        acceptNullTuple($array);
    } else {
        acceptString($array[0]);
    }
}

/** @param list{string, 1?: ?string} $array */
function optionalNullableTupleEntry(array $array): void
{
    if (!isset($array[1])) {
        acceptOptionalNullTuple($array);

        // @mago-expect analysis:possibly-undefined-int-array-index
        acceptNull($array[1]);

        if (array_key_exists(1, $array)) {
            acceptNull($array[1]);
        }
    }
}

/** @param array{value: mixed, label: string} $array */
function requiredMixedKey(array $array): void
{
    if (!isset($array['value'])) {
        acceptNull($array['value']);
        acceptRequiredNull($array);
    }
}

/**
 * @template T of string|null
 * @param array{value: T, label: string} $array
 */
function requiredTemplateKey(array $array): void
{
    if (!isset($array['value'])) {
        acceptNull($array['value']);
        acceptRequiredNull($array);
    }
}

/** @param array{inner: array{value: ?string, label: string}} $array */
function nestedNullableKey(array $array): void
{
    if (!isset($array['inner']['value'])) {
        acceptNull($array['inner']['value']);
        acceptRequiredNull($array['inner']);
    }
}

/** @param array{value: ?string, label: string} $array */
function nullableKeyGuard(array $array): string
{
    if (!isset($array['value'])) {
        acceptRequiredNull($array);

        return '';
    }

    return $array['value'];
}

/** @param array{value?: ?string, label: string} $array */
function absentAfterNullableGuard(array $array): void
{
    if (!isset($array['value']) && !array_key_exists('value', $array)) {
        // @mago-expect analysis:undefined-string-array-index
        acceptNull($array['value']);
    }
}

/**
 * @template T of string
 * @param array{value: T, label: string} $array
 */
function requiredNonNullTemplateKey(array $array): void
{
    // @mago-expect analysis:impossible-condition
    if (!isset($array['value'])) {
    }
}

/** @param array{value?: non-empty-mixed, label: string} $array */
function absentNonEmptyMixedKey(array $array): void
{
    if (!isset($array['value'])) {
        // @mago-expect analysis:undefined-string-array-index
        acceptNull($array['value']);
    }
}

/**
 * @param array{value: int, tag: 'number'}|array{value: null, tag: 'null'} $array
 * @return array{value: null, tag: 'null'}|null
 */
function nullableUnionEntry(array $array): ?array
{
    if (!isset($array['value'])) {
        return $array;
    }

    return null;
}

/**
 * @param list{int, 'number'}|list{null, 'null'} $array
 * @return list{null, 'null'}|null
 */
function nullableUnionTupleEntry(array $array): ?array
{
    if (!isset($array[0])) {
        return $array;
    }

    return null;
}

/**
 * @param array{value: ?int, tag: 'present'}|array{tag: 'absent'} $array
 * @return array{tag: 'absent'}|null
 */
function absentUnionEntry(array $array): ?array
{
    if (!array_key_exists('value', $array)) {
        return $array;
    }

    return null;
}
