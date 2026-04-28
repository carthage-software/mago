<?php

declare(strict_types=1);

function takesBool(bool $b): bool { return $b; }

$a = NAN == NAN;
$b = NAN < 5;
$c = NAN > 5;
takesBool($a);
takesBool($b);
takesBool($c);
