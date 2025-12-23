<?php

/**
 * @mago-expect analysis:redundant-null-coalesce
 */
function testRedundantNullCoalesce(string $value): string
{
    return $value ?? 'default';
}
