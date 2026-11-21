<?php

declare(strict_types=1);

function date_interval_fields_are_writable(DateInterval $interval): DateInterval
{
    $interval->invert = 1;
    $interval->d = 3;

    return $interval;
}
