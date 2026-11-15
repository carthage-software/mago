<?php

declare(strict_types=1);

function probe(string $pattern, string $s): string
{
    /**
     * @mago-expect analysis:nullable-return-statement
     * @mago-expect analysis:invalid-return-statement
     */
    return preg_replace($pattern, 'bar', $s);
}
