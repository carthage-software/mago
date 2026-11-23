<?php

declare(strict_types=1);

function timezone_names(DateTimeImmutable $immutable, DateTime $mutable, DateTimeInterface $date): string
{
    return $immutable->getTimezone()->getName() . $mutable->getTimezone()->getName() . $date->getTimezone()->getName();
}

function checked_timezone(DateTimeImmutable $immutable, DateTime $mutable, DateTimeInterface $date): string
{
    $timezone = $immutable->getTimezone();
    if ($timezone === false) {
        return '';
    }

    if ($mutable->getTimezone() === false || !$date->getTimezone()) {
        return '';
    }

    return $timezone->getName();
}
