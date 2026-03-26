<?php

declare(strict_types=1);

/** @var array{foo: int, ...}|array{foo: int} $x */

// @mago-expect analysis:undefined-string-array-index
var_dump($x['bar']);

// This used to produce a bogus impossible-nonnull-entry-check
isset($x['bar']);
