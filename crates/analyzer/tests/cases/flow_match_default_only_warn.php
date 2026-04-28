<?php

declare(strict_types=1);

/**
 * @mago-expect analysis:match-expression-only-default-arm
 */
function flow_match_default_only_warn(int $v): string
{
    return match ($v) {
        default => 'always',
    };
}
