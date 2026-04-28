<?php

declare(strict_types=1);

function flow_try_multi_catch_pipe(): string
{
    try {
        throw new \RuntimeException('boom');
    } catch (\RuntimeException | \LogicException $e) {
        return $e->getMessage();
    }
}
