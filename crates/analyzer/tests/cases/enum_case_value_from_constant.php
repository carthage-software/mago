<?php

declare(strict_types=1);

const HIGH = 20;

final class Limits
{
    public const MID = 10;
}

enum Level: int
{
    case Low = 1;
    case Mid = Limits::MID;
    case High = HIGH;
}

/** @param value-of<Level> $value */
function describe(int $value): string
{
    return Level::tryFrom($value)?->name ?? '';
}

function run(Level $level): string
{
    return describe($level->value) . describe(10);
}

final class Keys
{
    /** @var array-key */
    public const ID = 'id';
}

enum Column: string
{
    case Id = Keys::ID;
}
