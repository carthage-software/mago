<?php

declare(strict_types=1);

final class SomeClass {}

final class OtherClass {}

function param_array_for_object(SomeClass $info = []): SomeClass
{
    return $info;
}

function param_int_for_string(string $s = 1): string
{
    return $s;
}

function param_string_for_int(int $i = 'x'): int
{
    return $i;
}

function param_float_for_int(int $i = 1.5): int
{
    return $i;
}

function param_int_for_bool(bool $b = 1): bool
{
    return $b;
}

function param_string_for_float(float $f = 'no'): float
{
    return $f;
}

function param_int_for_array(array $a = 1): array
{
    return $a;
}

function param_wrong_object(SomeClass $c = new OtherClass()): SomeClass
{
    return $c;
}

// --- Invalid cases: docblock narrowing on scalars ---

/** @param non-empty-string $x */
function param_empty_for_non_empty_string(string $x = ''): string
{
    return $x;
}

/** @param positive-int $n */
function param_zero_for_positive_int(int $n = 0): int
{
    return $n;
}

/** @param int<1, 10> $n */
function param_out_of_range(int $n = 0): int
{
    return $n;
}

/** @param list<string> $a */
function param_non_list_for_list(array $a = ['k' => 'v']): array
{
    return array_values($a);
}

function param_valid_int(int $i = 0): int
{
    return $i;
}

function param_valid_string(string $s = 'hello'): string
{
    return $s;
}

function param_valid_empty_string(string $s = ''): string
{
    return $s;
}

function param_valid_nullable(?string $s = null): ?string
{
    return $s;
}

function param_valid_union(int|string $v = 'x'): int|string
{
    return $v;
}

function param_valid_union_int(int|string $v = 42): int|string
{
    return $v;
}

function param_valid_bool(bool $b = true): bool
{
    return $b;
}

function param_valid_float(float $f = 1.5): float
{
    return $f;
}

function param_valid_int_for_float(float $f = 1): float
{
    return $f;
}

/** @param non-empty-string $s */
function param_valid_non_empty_string(string $s = 'x'): string
{
    return $s;
}

/** @param positive-int $n */
function param_valid_positive_int(int $n = 1): int
{
    return $n;
}

function param_valid_empty_array(array $a = []): array
{
    return $a;
}

/** @param list<int> $a */
function param_valid_list(array $a = [1, 2, 3]): array
{
    return $a;
}

/** @param list<int> $a */
function param_valid_empty_list(array $a = []): array
{
    return $a;
}

// Variadic parameters: default is implicit empty array - no check.
function param_valid_variadic(int ...$ints): int
{
    return array_sum($ints);
}

class InvalidConstants
{
    public const string WRONG_INT_FOR_STRING = 12;

    public const int WRONG_STRING_FOR_INT = 'hello';

    public const bool WRONG_INT_FOR_BOOL = 1;

    public const float WRONG_STRING_FOR_FLOAT = 'no';

    public const array WRONG_INT_FOR_ARRAY = 1;
}

class InvalidDocblockConstants
{
    /** @var non-empty-string */
    public const string EMPTY_FOR_NON_EMPTY = '';

    /** @var positive-int */
    public const int ZERO_FOR_POSITIVE = 0;

    /** @var list<int> */
    public const array ASSOC_FOR_LIST = ['k' => 1];
}

class ValidConstants
{
    public const string OK_STRING = 'hello';
    public const int OK_INT = 42;
    public const bool OK_BOOL = true;
    public const float OK_FLOAT = 1.5;
    public const float OK_INT_FOR_FLOAT = 1;
    public const array OK_ARRAY = [1, 2, 3];

    /** @var non-empty-string */
    public const string OK_NON_EMPTY = 'x';

    /** @var positive-int */
    public const int OK_POSITIVE = 1;
}

class InvalidProperties
{
    public string $int_for_string = 1;

    public int $string_for_int = 'hi';

    public bool $int_for_bool = 1;

    public float $string_for_float = 'no';

    public array $int_for_array = 1;

    public ?SomeClass $array_for_nullable_object = [];
}

class InvalidDocblockProperties
{
    /** @var non-empty-string */
    public string $empty_for_non_empty = '';

    /** @var positive-int */
    public int $zero_for_positive = 0;

    /** @var list<int> */
    public array $assoc_for_list = ['k' => 1];
}

class ValidProperties
{
    public string $ok_string = 'hello';
    public string $ok_empty = '';
    public int $ok_int = 42;
    public bool $ok_bool = true;
    public float $ok_float = 1.5;
    public float $ok_int_for_float = 1;
    public array $ok_array = [];
    public ?string $ok_nullable_string = null;
    public ?SomeClass $ok_nullable_object = null;
    public int|string $ok_union_int = 0;
    public int|string $ok_union_string = '';

    /** @var non-empty-string */
    public string $ok_non_empty = 'x';

    /** @var positive-int */
    public int $ok_positive = 1;

    /** @var list<int> */
    public array $ok_list = [1, 2, 3];

    /** @var list<int> */
    public array $ok_empty_list = [];
}

class InvalidPromoted
{
    public function __construct(
        public string $promoted_int_for_string = 1,
    ) {}
}

class InvalidPromotedDocblock
{
    /** @param non-empty-string $promoted */
    public function __construct(
        public string $promoted = '',
    ) {}
}

class ValidPromoted
{
    public function __construct(
        public string $promoted_ok = 'hi',
        public ?int $promoted_nullable = null,
        public int|string $promoted_union = 'x',
    ) {}
}
