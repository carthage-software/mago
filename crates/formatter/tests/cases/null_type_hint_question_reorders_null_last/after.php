<?php

function nullable_question(?string $a): ?int
{
    return 1;
}

function null_first_single(?string $a): ?int
{
    return 1;
}

function null_last_single(?string $a): ?int
{
    return 1;
}

function null_first_multi(int|string|null $a): bool|int|null
{
    return 1;
}

function null_middle_multi(int|string|null $a): bool|int|null
{
    return 1;
}

function null_last_multi(int|string|null $a): int|string|null
{
    return 1;
}

class Foo
{
    public ?string $nullable;
    public ?string $nullFirst;
    public ?string $nullLast;
    public int|string|null $nullFirstMulti;
    public int|string|null $nullMiddleMulti;
}
