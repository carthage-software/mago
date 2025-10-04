<?php

final class Temporal
{
    public function __construct(
        private string $datetime,
    ) {}

    public static function createFromFormat(string $format, string $datetime): static
    {
        return new static($format . $datetime);
    }

    public static function createFromInterface(Temporal $temporal): static
    {
        return new static($temporal->datetime);
    }

    public function format(string $format): string
    {
        return $format . $this->datetime;
    }
}

function maybeUseDateAndTime(null|Temporal $date, null|Temporal $time): void
{
    match (true) {
        null !== $date && null !== $time => useDateAndTime($date, $time),
        null !== $date => useDate($date),
        null !== $time => useTime($time),
        default => null,
    };
}

function maybeUseDateAndTimeUsingIf(null|Temporal $date, null|Temporal $time): void
{
    if (null !== $date && null !== $time) {
        useDateAndTime($date, $time);
    } elseif (null !== $date) {
        useDate($date);
    } elseif (null !== $time) {
        useTime($time);
    }
}

/**
 * @mago-expect analysis:unhandled-thrown-type
 */
function maybeUseDateAndTimeNonExhaustive(null|Temporal $date, null|Temporal $time): void
{
    match (true) { // @mago-expect analysis:match-not-exhaustive
        null !== $date && null !== $time => useDateAndTime($date, $time),
        null !== $date => useDate($date),
    };
}

function maybeUseDateAndTimeNonExhaustiveUsingIf(null|Temporal $date, null|Temporal $time): void
{
    if (null !== $date && null !== $time) {
        useDateAndTime($date, $time);
    } elseif (null !== $date) {
        useDate($date);
    }
}

function useDateAndTime(Temporal $date, Temporal $time): void
{
    useDate($date);
    useTime($time);
}

function useTime(Temporal $time): void
{
    echo $time->format('H:i:s');
}

function useDate(Temporal $date): void
{
    echo $date->format('Y-m-d');
}
