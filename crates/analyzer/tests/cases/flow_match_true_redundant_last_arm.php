<?php

declare(strict_types=1);

function flow_match_arm_narrowing(int|string $v): string
{
    return match (true) {
        is_int($v) => 'i:' . $v,
        // @mago-expect analysis:redundant-type-comparison
        is_string($v) => $v,
    };
}
