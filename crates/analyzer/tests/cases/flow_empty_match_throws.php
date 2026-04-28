<?php

declare(strict_types=1);

/**
 * @mago-expect analysis:empty-match-expression
 * @mago-expect analysis:unhandled-thrown-type
 */
function flow_empty_match_throws(int $v): never
{
    match ($v) {};
}
