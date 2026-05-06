<?php

declare(strict_types=1);

/**
 */
function flow_empty_match_throws(int $v): never
{
    match ($v) { };
}
