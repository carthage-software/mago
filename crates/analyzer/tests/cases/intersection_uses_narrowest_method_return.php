<?php

declare(strict_types=1);

interface PlainString
{
    public function __toString(): string;
}

interface NumericString
{
    /** @return numeric-string */
    public function __toString(): string;
}

/** @param numeric-string $value */
function takes_numeric(string $value): void
{
    echo $value;
}

function narrowed(NumericString $value): void
{
    if ($value instanceof PlainString) {
        takes_numeric((string) $value);
    }
}

/** @param NumericString&PlainString $value */
function numeric_first(object $value): void
{
    takes_numeric((string) $value);
}

/** @param PlainString&NumericString $value */
function plain_first(object $value): void
{
    takes_numeric((string) $value);
}
interface HasId
{
    public function id(): int|string;
}

interface HasIntId
{
    public function id(): int;
}

function narrowed_id(HasIntId $value): int
{
    if ($value instanceof HasId) {
        return $value->id();
    }

    return 0;
}

/** @param HasIntId&HasId $value */
function declared_id(object $value): int
{
    return $value->id();
}
