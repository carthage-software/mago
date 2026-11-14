<?php

/**
 * @mago-expect analysis:redundant-condition
 * @mago-expect analysis:redundant-comparison,always-matching-switch-case,unreachable-switch-case,unreachable-switch-default
 */
function test_switch_logically_unreachable_case(int $value): string
{
    switch (true) {
        case $value > 0:
            return 'positive';
        case $value <= 0:
            return 'not positive';
        case $value === 5:
            return 'five';
        default:
            return 'other';
    }
}
