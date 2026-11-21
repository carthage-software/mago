<?php

declare(strict_types=1);

namespace NamespacedPlatformConstant;

function namespaced_platform_constant_keeps_platform_type(): string
{
    if (PHP_INT_SIZE === 4) {
        return '32-bit';
    }

    return '64-bit';
}
