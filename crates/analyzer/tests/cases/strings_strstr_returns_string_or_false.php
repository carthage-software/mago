<?php

declare(strict_types=1);

function probe(string $h): string
{
    /**
     * @mago-expect analysis:falsable-return-statement
     * @mago-expect analysis:invalid-return-statement
     */
    return strstr($h, 'foo');
}
