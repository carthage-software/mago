<?php

declare(strict_types=1);

$range = new DatePeriod(new DateTime('2011-01-01'), new DateInterval('P1D'), new DateTime('2011-12-31'));
foreach ($range as $date) {
    $date->format('N');
}
