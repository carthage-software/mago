<?php

declare(strict_types=1);

final class FailBR extends RuntimeException
{
}

/**
 * @throws FailBR
 */
function lowerBR(): void
{
    throw new FailBR();
}

/**
 * @throws FailBR
 */
function upperBR(): void
{
    lowerBR();
}

try {
    upperBR();
} catch (FailBR) {
}
