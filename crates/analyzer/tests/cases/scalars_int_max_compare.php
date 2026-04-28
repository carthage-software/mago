<?php

declare(strict_types=1);

function takesBool(bool $b): bool { return $b; }

$a = PHP_INT_MAX > 1;
takesBool($a);
