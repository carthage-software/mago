<?php

declare(strict_types=1);

/** @var array{foo: int, ...}|array{foo: int} $x */

// @mago-expect analysis:possibly-undefined-string-array-index
var_dump($x['bar']);
