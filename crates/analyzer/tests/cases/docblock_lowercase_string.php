<?php

declare(strict_types=1);

/** @param lowercase-string $s */
function takeLowerAQ(string $s): string
{
    return $s;
}

takeLowerAQ('hello');
takeLowerAQ('');
/** @mago-expect analysis:invalid-argument */
takeLowerAQ('Hello');
