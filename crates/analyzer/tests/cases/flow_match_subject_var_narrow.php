<?php

declare(strict_types=1);

function flow_match_subject_var_narrow(int|string $v): string
{
    return match (true) {
        is_int($v) => (string) $v,
        default => $v,
    };
}
