<?php

declare(strict_types=1);

/**
 */
function flow_throw_unhandled(): never
{
    throw new \RuntimeException('boom');
}
