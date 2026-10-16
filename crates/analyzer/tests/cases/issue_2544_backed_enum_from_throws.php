<?php

declare(strict_types=1);

enum Issue2544Status: string
{
    case Open = 'open';
    case Closed = 'closed';
}

enum Issue2544Level: int
{
    case Low = 1;
    case High = 2;
}

function fromStatusLiteral(): Issue2544Status
{
    return Issue2544Status::from('open');
}

function fromLevelLiteral(): Issue2544Level
{
    return Issue2544Level::from(1);
}

/**
 * @param 'open'|'closed' $value
 */
function fromStatusValidUnion(string $value): Issue2544Status
{
    return Issue2544Status::from($value);
}

/**
 * @param value-of<Issue2544Status> $value
 */
function fromStatusValueOf(string $value): Issue2544Status
{
    return Issue2544Status::from($value);
}

/**
 * @param value-of<Issue2544Status> $value
 */
function fromStatusValueOfNamedArgument(string $value): Issue2544Status
{
    return Issue2544Status::from(value: $value);
}

/**
 * @param value-of<Issue2544Level> $value
 */
function fromLevelValueOf(int $value): Issue2544Level
{
    return Issue2544Level::from($value);
}

/**
 * @param key-of<array<value-of<Issue2544Status>, int>> $key
 */
function fromStatusKeyOf(string $key): Issue2544Status
{
    return Issue2544Status::from($key);
}

/**
 * @param array<value-of<Issue2544Status>, int> $counts
 */
function fromStatusArrayKey(array $counts, string $key): ?Issue2544Status
{
    if (!isset($counts[$key])) {
        return null;
    }

    return Issue2544Status::from($key);
}

/**
 * @param array<'open'|'closed', int> $first
 * @param array<'open'|'other', int> $second
 */
function fromStatusMultipleArrayKeys(array $first, array $second, string $key): ?Issue2544Status
{
    if (isset($first[$key], $second[$key])) {
        return Issue2544Status::from($key);
    }

    return null;
}

/** @implements ArrayAccess<string, int> */
final class Issue2544ArrayAccess implements ArrayAccess
{
    public function offsetExists(mixed $offset): bool
    {
        return true;
    }

    public function offsetGet(mixed $offset): mixed
    {
        return 0;
    }

    public function offsetSet(mixed $offset, mixed $value): void
    {
    }

    public function offsetUnset(mixed $offset): void
    {
    }
}

/**
 * @param array<value-of<Issue2544Status>, int>|Issue2544ArrayAccess $counts
 * @mago-expect analysis:unhandled-thrown-type
 */
function fromPossiblyArrayAccess(array|Issue2544ArrayAccess $counts, string $key): ?Issue2544Status
{
    if (!isset($counts[$key])) {
        return null;
    }

    return Issue2544Status::from($key);
}

/**
 * @mago-expect analysis:unhandled-thrown-type
 */
function fromStatusUnknownString(string $value): Issue2544Status
{
    return Issue2544Status::from($value);
}

/**
 * @mago-expect analysis:unhandled-thrown-type
 */
function fromLevelUnknownInt(int $value): Issue2544Level
{
    return Issue2544Level::from($value);
}

/**
 * @mago-expect analysis:unhandled-thrown-type
 */
function fromStatusInvalidLiteral(): Issue2544Status
{
    return Issue2544Status::from('missing');
}

/**
 * @param 'open'|'missing' $value
 * @mago-expect analysis:unhandled-thrown-type
 */
function fromStatusPartiallyValidUnion(string $value): Issue2544Status
{
    return Issue2544Status::from($value);
}
