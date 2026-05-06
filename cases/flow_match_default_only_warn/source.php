<?php

declare(strict_types=1);

/**
 */
function flow_match_default_only_warn(int $v): string
{
    return match ($v) { default => 'always' };
}
