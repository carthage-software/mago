<?php

declare(strict_types=1);

/** @param non-empty-string $s */
function require_non_empty(string $s): void
{
    echo $s;
}

require_non_empty('');
