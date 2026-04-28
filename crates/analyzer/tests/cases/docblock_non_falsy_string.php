<?php

declare(strict_types=1);

/** @param non-falsy-string $s */
function takeNonFalsyBG(string $s): string
{
    return $s;
}

takeNonFalsyBG('hello');
takeNonFalsyBG('123');
/** @mago-expect analysis:invalid-argument */
takeNonFalsyBG('');
