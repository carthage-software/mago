<?php

declare(strict_types=1);

/**
 * @psalm-inheritors Monday|Tuesday|Wednesday|Thursday|Friday|Saturday|Sunday
 */
abstract class DayOfWeek
{
    abstract public function getName(): string;
}

class Monday extends DayOfWeek
{
    public function getName(): string
    {
        return 'Monday';
    }

    public function isWeekday(): bool
    {
        return true;
    }
}

class Tuesday extends DayOfWeek
{
    public function getName(): string
    {
        return 'Tuesday';
    }

    public function isWeekday(): bool
    {
        return true;
    }
}

class Wednesday extends DayOfWeek
{
    public function getName(): string
    {
        return 'Wednesday';
    }

    public function isWeekday(): bool
    {
        return true;
    }
}

class Thursday extends DayOfWeek
{
    public function getName(): string
    {
        return 'Thursday';
    }

    public function isWeekday(): bool
    {
        return true;
    }
}

class Friday extends DayOfWeek
{
    public function getName(): string
    {
        return 'Friday';
    }

    public function isWeekday(): bool
    {
        return true;
    }
}

class Saturday extends DayOfWeek
{
    public function getName(): string
    {
        return 'Saturday';
    }

    public function isWeekend(): bool
    {
        return true;
    }
}

class Sunday extends DayOfWeek
{
    public function getName(): string
    {
        return 'Sunday';
    }

    public function isWeekend(): bool
    {
        return true;
    }
}

function checkDay(DayOfWeek $day): Sunday|string
{
    if ($day instanceof Monday) {
        return $day->isWeekday() ? 'Weekday: Monday' : 'Not weekday';
    }

    if ($day instanceof Tuesday) {
        return $day->isWeekday() ? 'Weekday: Tuesday' : 'Not weekday';
    }

    if ($day instanceof Wednesday) {
        return $day->isWeekday() ? 'Weekday: Wednesday' : 'Not weekday';
    }

    if ($day instanceof Thursday) {
        return $day->isWeekday() ? 'Weekday: Thursday' : 'Not weekday';
    }

    if ($day instanceof Friday) {
        return $day->isWeekday() ? 'Weekday: Friday' : 'Not weekday';
    }

    if ($day instanceof Saturday) {
        return $day->isWeekend() ? 'Weekend: Saturday' : 'Not weekend';
    }

    // Sunday
    return $day;
}
