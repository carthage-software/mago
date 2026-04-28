<?php

declare(strict_types=1);

/**
 * @mago-expect analysis:unhandled-thrown-type
 */
function flow_throw_unhandled(): never
{
    throw new \RuntimeException('boom');
}
