<?php

declare(strict_types=1);

/** @param truthy-string $s */
function takeTruthyBF(string $s): string
{
    return $s;
}

takeTruthyBF('hello');
/** @mago-expect analysis:invalid-argument */
takeTruthyBF('');
