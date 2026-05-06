<?php

declare(strict_types=1);

/**
 * @param array{a?: array{b?: array{c?: int}}} $a
 */
function flow_isset_chain_then_use(array $a): int
{
    if (isset($a['a']['b']['c'])) {
        return $a['a']['b']['c'] + 1;
    }

    return 0;
}
