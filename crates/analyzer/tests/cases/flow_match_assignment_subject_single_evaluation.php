<?php

declare(strict_types=1);

/**
 * @mago-expect analysis:mixed-argument(1)
 */
function flow_match_assignment_subject_single_evaluation(mixed $value): string
{
    return match ($length = strlen($value)) {
        0 => 'empty',
        1 => 'one',
        default => 'many ' . $length,
    };
}
