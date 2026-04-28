<?php

declare(strict_types=1);

/** @param non-empty-string $s */
function neStringAJ(string $s): string
{
    return $s;
}

neStringAJ('hello');
/** @mago-expect analysis:invalid-argument */
neStringAJ('');
