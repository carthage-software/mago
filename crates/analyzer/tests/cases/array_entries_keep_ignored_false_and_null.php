<?php

declare(strict_types=1);

/** @psalm-ignore-nullable-return */
function maybeName(): ?string
{
    return rand(0, 1) ? 'a' : null;
}

/** @param array{tz: DateTimeZone} $a */
function takesTz(array $a): void {}

/** @param array{id: int, ...<string, string>} $a */
function takesShape(array $a): void {}

function f(DateTimeImmutable $d): void
{
    takesTz(['tz' => $d->getTimezone()]);
    takesShape(['id' => 1, 'name' => maybeName()]);
}

/** @param list<DateTimeZone> $l */
function takes_list(array $l): void {}

function g(DateTimeImmutable $d): void
{
    takes_list([$d->getTimezone()]);
}
