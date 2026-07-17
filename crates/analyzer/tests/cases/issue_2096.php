<?php

declare(strict_types=1);

/** @var mixed $array */
$array = [];

if (
    isset($array['test'])
    || is_string(
        // @mago-expect analysis:mixed-array-access
        $array['test'],
    )
) {
}

/** @var mixed $guarded */
$guarded = [];

if (isset($guarded['test']) && is_string($guarded['test'])) {
}
