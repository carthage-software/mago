<?php

declare(strict_types=1);

/** @var callable-string $fn */
$fn = 'strlen';

$closure = Closure::fromCallable($fn);
$closure('hello');
