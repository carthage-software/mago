<?php

declare(strict_types=1);

/**
 * @throws RuntimeException
 */
function throwerBQ(): void
{
    throw new RuntimeException('boom');
}

try {
    throwerBQ();
} catch (RuntimeException) {
}
