<?php

declare(strict_types=1);

/** @param non-empty-string $s */
function require_non_empty(string $s): void
{
    echo $s;
}

/** @mago-expect analysis:invalid-argument */
require_non_empty('');
