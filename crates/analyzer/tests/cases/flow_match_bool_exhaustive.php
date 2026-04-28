<?php

declare(strict_types=1);

function flow_match_bool_exhaustive(bool $b): string
{
    return match ($b) {
        true => 't',
        false => 'f',
    };
}
