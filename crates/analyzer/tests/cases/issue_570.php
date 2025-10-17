<?php

declare(strict_types=1);

/**
 * @param array{ foo?: array{ bar?: string} } $y
 */
function x1(array $y): void
{
    if (isset($y['foo']) && !isset($y['foo']['bar'])) {
        echo $y['foo']['bar']; // @mago-expect analysis:possibly-undefined-string-array-index
    }
}

/**
 * @param array{ foo?: array{ bar?: string} } $y
 */
function x2(array $y): void
{
    if (!isset($y['foo']['bar']) && isset($y['foo'])) {
        echo $y['foo']['bar']; // @mago-expect analysis:possibly-undefined-string-array-index
    }
}
