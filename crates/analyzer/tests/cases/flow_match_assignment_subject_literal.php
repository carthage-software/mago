<?php

declare(strict_types=1);

/**
 * @param 'a'|'b'|'c' $letter
 */
function flow_match_assignment_subject_literal(string $letter): int
{
    return match ($copy = $letter) {
        'a' => 1,
        'b' => 2,
        'c' => strlen($copy),
    };
}

function flow_match_assignment_subject_bool(bool $flag): string
{
    return match ($copy = $flag) {
        true => 'yes',
        false => 'no',
    };
}
