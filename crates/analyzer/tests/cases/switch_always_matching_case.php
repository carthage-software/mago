<?php

function test_switch_always_matching_case(int $value): string
{
    switch (true) {
        // @mago-expect analysis:redundant-type-comparison,always-matching-switch-case
        case is_int($value):
            return 'is int';
        // @mago-expect analysis:unreachable-switch-case
        case $value > 0:
            return 'is positive';
        // @mago-expect analysis:unreachable-switch-default
        default:
            return 'other';
    }
}
