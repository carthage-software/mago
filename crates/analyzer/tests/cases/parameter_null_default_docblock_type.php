<?php

declare(strict_types=1);

/**
 * A `null` default adds `null` to a docblock type, as it does to a native type.
 *
 * @param string $value
 */
function docblock_null_default($value = null): string
{
    if ($value === null) {
        return 'none';
    }

    return $value;
}

/**
 * @param int $value
 */
function docblock_null_default_coalesce($value = NULL): int
{
    return $value ?? 0;
}

/**
 * @param string $value
 */
function docblock_null_default_unchecked($value = null): int
{
    // @mago-expect analysis:possibly-null-argument
    return strlen($value);
}

/**
 * @param string $value
 */
function docblock_string_default($value = 'none'): string
{
    return $value;
}

function docblock_null_default_caller(): void
{
    echo docblock_null_default(null);
    echo docblock_null_default_coalesce(null);
    // @mago-expect analysis:null-argument
    echo docblock_string_default(null);
}

/**
 * @template T of object
 *
 * @param T $object
 *
 * @return T|null
 */
function docblock_null_default_template($object = null): ?object
{
    return $object;
}

interface DocblockNullDefaultParent
{
    /**
     * @param string $value
     */
    public function optional($value = null): void;

    /**
     * @param string $value
     */
    public function required($value): void;
}

final class DocblockNullDefaultChild implements DocblockNullDefaultParent
{
    /**
     * {@inheritdoc}
     */
    public function optional($value = null): void
    {
        echo $value ?? 'none';
    }

    /**
     * The inherited type gets the `null` of this method's own default.
     */
    public function required($value = null): void
    {
        echo $value ?? 'none';
    }
}

final class DocblockNullDefaultNarrowed implements DocblockNullDefaultParent
{
    /**
     * @param string $value
     *
     * @mago-expect analysis:incompatible-parameter-type
     */
    public function optional($value = 'none'): void
    {
        echo $value;
    }

    /**
     * @param string $value
     */
    public function required($value): void
    {
        echo $value;
    }
}
