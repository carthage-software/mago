<?php

declare(strict_types=1);

/** @param int $x */
function narrow_scalar_union_to_int(int|string $x): void // @mago-expect analysis:docblock-parameter-narrowing
{
    echo $x;
}

/** @param string $x */
function narrow_scalar_union_to_string(int|string $x): void // @mago-expect analysis:docblock-parameter-narrowing
{
    echo $x;
}

/** @param int $x */
function narrow_triple_union_to_int(int|string|float $x): void // @mago-expect analysis:docblock-parameter-narrowing
{
    echo $x;
}

/** @param array<string, mixed> $o */
function narrow_int_string_array_to_array(int|string|array $o): int // @mago-expect analysis:docblock-parameter-narrowing
{
    return mt_rand();
}

/** @param int $x */
function narrow_int_or_false_to_int(int|false $x): void // @mago-expect analysis:docblock-parameter-narrowing
{
    echo $x;
}

/** @param string $x */
function narrow_string_or_false_to_string(string|false $x): void // @mago-expect analysis:docblock-parameter-narrowing
{
    echo $x;
}

/** @param Foo $x */
function narrow_object_union_to_single(Foo|Bar $x): void // @mago-expect analysis:docblock-parameter-narrowing
{
    echo get_class($x);
}

class Foo
{
    /** @param int $x */
    public function method_narrow_union(int|string $x): void // @mago-expect analysis:docblock-parameter-narrowing
    {
        echo $x;
    }

    /** @param int $x */
    public static function static_narrow_union(int|string $x): void // @mago-expect analysis:docblock-parameter-narrowing
    {
        echo $x;
    }
}

class Bar
{
}

$closure_narrowing =
    /** @param int $x */
    static function (int|string $x): void { // @mago-expect analysis:docblock-parameter-narrowing
        echo $x;
    };

// @mago-expect analysis:docblock-parameter-narrowing
$arrow_narrowing =
    /** @param int $x */
    static fn(int|string $x): int => $x + 1;

/** @param non-empty-string $x */
function refine_string(string $x): void
{
    echo $x;
}

/** @param positive-int $x */
function refine_int(int $x): void
{
    echo $x;
}

/** @param 1|2|3 $x */
function refine_int_to_literals(int $x): void
{
    echo $x;
}

/** @param 'a'|'b' $x */
function refine_string_to_literals(string $x): void
{
    echo $x;
}

/** @param array<string, int> $x */
function refine_array_shape(array $x): void
{
    foreach ($x as $k => $v) {
        echo "$k=$v";
    }
}

/** @param list<int> $x */
function refine_array_to_list(array $x): void
{
    foreach ($x as $v) {
        echo $v;
    }
}

/** @param non-empty-list<int> $x */
function refine_array_to_non_empty_list(array $x): void
{
    echo $x[0];
}

/** @param non-empty-string|null $x */
function refine_one_branch_of_nullable(?string $x): void
{
    echo $x ?? 'default';
}

/** @param array<int>|null $x */
function refine_nullable_array(array|null $x): void
{
    if ($x !== null) {
        foreach ($x as $v) {
            echo $v;
        }
    }
}

/** @param int $x */
function refine_mixed_to_int(mixed $x): void
{
    echo $x;
}

/** @param list<string> $x */
function refine_mixed_to_list(mixed $x): void
{
    foreach ($x as $s) {
        echo $s;
    }
}

/** @param int|string $x */
function docblock_equals_native(int|string $x): void
{
    echo $x;
}

/** @param int|string|float $x */
function docblock_widens_float_to_float(int|string|float $x): void
{
    echo $x;
}

/**
 * @template K of array-key
 * @template V
 *
 * @param K $key
 * @param V $value
 * @return array<K, V>
 */
function template_param_skipped(string|int $key, mixed $value): array
{
    return [$key => $value];
}

/**
 * @template T of object
 *
 * @param T $item
 */
function template_object_skipped(object $item): void
{
    echo get_class($item);
}

interface Animal
{
}

class Dog implements Animal
{
}

/** @param Dog $x */
function refine_interface_to_class(Animal $x): void
{
    echo get_class($x);
}

class Collection
{
    /**
     * @param non-empty-string $key
     * @param list<int> $values
     */
    public function method_refine(string $key, array $values): void
    {
        echo "$key: " . count($values);
    }
}
