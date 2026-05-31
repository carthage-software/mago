<?php

function nullable_question(?string $a): ?int
{
    return 1;
}

function null_first_single(null|string $a): null|int
{
    return 1;
}

function null_last_single(string|null $a): int|null
{
    return 1;
}

function null_first_multi(null|int|string $a): null|bool|int
{
    return 1;
}

function null_middle_multi(int|null|string $a): bool|null|int
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
    public null|string $nullFirst;
    public string|null $nullLast;
    public null|int|string $nullFirstMulti;
    public int|null|string $nullMiddleMulti;
}
