<?php

declare(strict_types=1);

$items = json_decode('[]', associative: true);

$r = array_filter($items, static fn(array $x): bool => !empty($x['name']));

var_dump($r);
