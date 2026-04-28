<?php

declare(strict_types=1);

/**
 * @param array{value?: int} $a
 */
function flow_isset_via_ternary_branch(array $a): int
{
    return $a['value'] ?? 0;
}
