<?php

/**
 * @param non-empty-list<string> $list
 * @return non-empty-list<int>
 */
function takesNonEmptyList(array $list): array
{
    return array_map(strlen(...), $list);
}