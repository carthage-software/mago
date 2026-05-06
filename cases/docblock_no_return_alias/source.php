<?php

declare(strict_types=1);

/**
 * @return no-return
 *
 * @throws RuntimeException
 */
function alwaysFailAW(): never
{
    throw new RuntimeException('boom');
}

try {
    alwaysFailAW();
} catch (RuntimeException) {
}
