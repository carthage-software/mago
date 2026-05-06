<?php

/**
 */
function testRedundantNullCoalesce(string $value): string
{
    return $value ?? 'default';
}
