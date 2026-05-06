<?php

declare(strict_types=1);

function flow_try_catch_specific(): string
{
    try {
        throw new \RuntimeException('rt');
    } catch (\RuntimeException $e) {
        return $e->getMessage();
    }
}
