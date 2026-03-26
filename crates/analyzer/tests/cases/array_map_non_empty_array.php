<?php

/**
 * @param non-empty-array<string, string> $arr
 * @return non-empty-array<string, int>
 */
function takesNonEmptyArray(array $arr): array
{
    return array_map(strlen(...), $arr);
}