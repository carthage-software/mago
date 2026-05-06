<?php

declare(strict_types=1);

/** @var array{foo: int, ...}|array{foo: int} $x */

var_dump($x['bar']);
