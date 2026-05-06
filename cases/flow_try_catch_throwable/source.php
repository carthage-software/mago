<?php

declare(strict_types=1);

function flow_try_catch_throwable(): string
{
    try {
        throw new \RuntimeException('boom');
    } catch (\Throwable $e) {
        return $e->getMessage();
    }
}
