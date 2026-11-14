<?php

declare(strict_types=1);

final class MyDate extends DateTime
{
}

final class MyImmutableDate extends DateTimeImmutable
{
}

function mutable(): MyDate|false
{
    return MyDate::createFromFormat('Y-m-d', '2024-01-01');
}

function immutable(): MyImmutableDate|false
{
    return MyImmutableDate::createFromFormat('Y-m-d', '2024-01-01');
}
