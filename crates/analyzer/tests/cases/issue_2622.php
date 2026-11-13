<?php

declare(strict_types=1);

namespace Issue2622;

use DateTime;
use DateTimeImmutable;
use DateTimeInterface;

interface Clock extends DateTimeInterface {}

final class FrozenClock extends DateTimeImmutable implements Clock {}

class MutableClock extends DateTime {}

function to_immutable(DateTimeInterface $date): DateTimeImmutable
{
    if ($date instanceof DateTime) {
        return DateTimeImmutable::createFromMutable($date);
    }

    return $date;
}

function to_mutable(DateTimeInterface $date): DateTime
{
    if (!$date instanceof DateTimeImmutable) {
        return $date;
    }

    return DateTime::createFromImmutable($date);
}

function nullable(?DateTimeInterface $date): ?DateTimeImmutable
{
    return $date instanceof DateTime ? DateTimeImmutable::createFromMutable($date) : $date;
}

function describe(DateTimeInterface $date): string
{
    return match (true) {
        $date instanceof DateTime => 'mutable',
        $date instanceof DateTimeImmutable => 'immutable',
    };
}

final readonly class Event
{
    public function __construct(
        public DateTimeInterface $at,
    ) {}

    public function at(): DateTimeImmutable
    {
        if ($this->at instanceof DateTime) {
            return DateTimeImmutable::createFromMutable($this->at);
        }

        return $this->at;
    }
}

function wrong_class(DateTimeInterface $date): DateTime
{
    if ($date instanceof DateTime) {
        return $date;
    }

    // @mago-expect analysis:invalid-return-statement
    return $date;
}

function subclass_ruled_out(DateTimeInterface $date): DateTimeImmutable
{
    if ($date instanceof MutableClock) {
        return DateTimeImmutable::createFromMutable($date);
    }

    // @mago-expect analysis:less-specific-return-statement
    return $date;
}

function user_interface(Clock $clock): Clock
{
    if ($clock instanceof DateTime) {
        return new FrozenClock();
    }

    return $clock;
}
