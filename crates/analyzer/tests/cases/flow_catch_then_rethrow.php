<?php

declare(strict_types=1);

/**
 * @throws \RuntimeException
 */
function flow_catch_then_rethrow(): never
{
    try {
        throw new \RuntimeException('inner');
    } catch (\RuntimeException $e) {
        throw $e;
    }
}
