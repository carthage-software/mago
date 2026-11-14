<?php

declare(strict_types=1);

/**
 * @param array<string, array<string, string>> $data
 * @return array<string, string>
 */
function &find_node(array &$data, string $path): array
{
    $data[$path] ??= [];
    $node = &$data[$path];

    return $node;
}
