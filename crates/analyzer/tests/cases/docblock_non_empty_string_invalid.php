<?php

declare(strict_types=1);

/** @param non-empty-string $s */
function neStringAK(string $s): string
{
    return $s;
}

/** @mago-expect analysis:invalid-argument */
neStringAK('');
