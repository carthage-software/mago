<?php

declare(strict_types=1);

/**
 * @return never
 *
 * @throws RuntimeException
 */
function alwaysFailAV(): never
{
    throw new RuntimeException('boom');
}

try {
    alwaysFailAV();
} catch (RuntimeException) {
}
