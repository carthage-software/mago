<?php

function test_switch_always_matching_case(int $value): string
{
    switch (true) {
        case is_int($value):
            return 'is int';
        case $value > 0:
            return 'is positive';
        default:
            return 'other';
    }
}
