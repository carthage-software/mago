<?php

declare(strict_types=1);

/**
 * @mago-expect analysis:unevaluated-code
 */
function nested_switch_break_two_exits_outer_switch(): void
{
    switch (mt_rand(0, 1)) {
        case 0:
            switch (mt_rand(0, 1)) {
                case 0:
                    break 2;
                default:
                    break 2;
            }

            throw new RuntimeException();
        default:
            return;
    }
}

/**
 * @mago-expect analysis:unevaluated-code
 */
function nested_switch_continue_two_exits_outer_switch(): void
{
    switch (mt_rand(0, 1)) {
        case 0:
            switch (mt_rand(0, 1)) {
                case 0:
                    continue 2;
                default:
                    continue 2;
            }

            throw new RuntimeException();
        default:
            return;
    }
}

/**
 * @mago-expect analysis:unevaluated-code
 */
function nested_switch_break_two_exits_outer_case_body(): int
{
    switch (mt_rand(0, 1)) {
        case 0:
            switch (mt_rand(0, 1)) {
                case 0:
                    break 2;
                default:
                    return 0;
            }

            throw new RuntimeException();
        default:
            return 0;
    }

    return 1;
}

function nested_switch_break_two_and_local_break_fallthrough(): int
{
    switch (mt_rand(0, 2)) {
        case 0:
            switch (mt_rand(0, 1)) {
                case 0:
                    break 2;
                default:
                    break;
            }
        case 1:
            return 0;
        default:
            return 0;
    }

    return 1;
}

/**
 * @mago-expect analysis:unevaluated-code
 */
function three_level_nested_switch_break_one_stays_in_innermost_switch(): int
{
    switch (mt_rand(0, 1)) {
        default:
            switch (mt_rand(0, 1)) {
                default:
                    switch (mt_rand(0, 1)) {
                        default:
                            break;
                    }

                    return 1;
            }
    }

    throw new RuntimeException();
}

/**
 * @mago-expect analysis:unevaluated-code
 */
function three_level_nested_switch_break_two_targets_middle_switch(): int
{
    switch (mt_rand(0, 1)) {
        default:
            switch (mt_rand(0, 1)) {
                default:
                    switch (mt_rand(0, 1)) {
                        default:
                            break 2;
                    }

                    throw new RuntimeException();
            }

            break;
    }

    return 1;
}

/**
 * @mago-expect analysis:unevaluated-code
 */
function three_level_nested_switch_break_three_targets_outer_switch(): int
{
    switch (mt_rand(0, 1)) {
        default:
            switch (mt_rand(0, 1)) {
                default:
                    switch (mt_rand(0, 1)) {
                        default:
                            break 3;
                    }
            }

            throw new RuntimeException();
    }

    return 1;
}

/**
 * @mago-expect analysis:unevaluated-code
 */
function switch_case_fallthrough_to_default_exits_switch(): void
{
    switch (mt_rand(0, 1)) {
        case 0:
            echo 'matched';
        default:
            return;
    }

    throw new RuntimeException();
}

/**
 * @mago-expect analysis:unevaluated-code
 */
function unreachable_duplicate_switch_case_does_not_poison_exit_summary(): void
{
    switch (mt_rand(0, 1)) {
        case 0:
            return;
        // @mago-expect analysis:unreachable-switch-case
        case 0:
            break;
        default:
            return;
    }

    throw new RuntimeException();
}

function finally_return_overrides_pending_break(): int
{
    while (true) {
        try {
            break;
        } finally {
            return 1;
        }
    }
}

/**
 * @mago-expect analysis:unevaluated-code
 */
function finally_normal_completion_preserves_pending_break(): int
{
    while (true) {
        try {
            break;
        } finally {
            echo 'cleanup';
        }

        echo 'unreachable';
    }

    return 1;
}

function finally_local_breaks_reach_finally_return(): int
{
    try {
        return 0;
    } finally {
        while (true) {
            switch (mt_rand(0, 1)) {
                case 0:
                    break;
                default:
                    break 2;
            }

            break;
        }

        return 1;
    }
}
