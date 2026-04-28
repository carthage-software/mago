<?php

declare(strict_types=1);

function flow_match_true_chain(int|string|float $v): string
{
    return match (true) {
        is_int($v) => 'int',
        is_float($v) => 'float',
        // @mago-expect analysis:redundant-type-comparison
        is_string($v) => $v,
    };
}
