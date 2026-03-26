<?php

// Nullable hint (?) should become T|null
function nullable_param(?string $a): ?int
{
    return 1;
}

// null|T union should become T|null
function null_first(null|string $a): null|\DateTimeImmutable
{
    return $a;
}

// T|null union should stay T|null
function null_last(string|null $a): \DateTimeImmutable|null
{
    return $a;
}

// Multi-type union: null|T1|T2 should become T1|T2|null
function null_first_multi(null|int|string $a): null|int|string
{
    return $a;
}

// Multi-type union: T1|null|T2 should become T1|T2|null
function null_middle_multi(int|null|string $a): int|null|string
{
    return $a;
}

// Multi-type union: T1|T2|null should stay T1|T2|null
function null_last_multi(int|string|null $a): int|string|null
{
    return $a;
}

class Foo
{
    // Property types
    public ?string $nullable;
    public null|string $nullFirst;
    public string|null $nullLast;
}
