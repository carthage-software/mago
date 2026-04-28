<?php

declare(strict_types=1);

function takesBool(bool $b): bool { return $b; }

$a = 0 == 'abc';
takesBool($a);
