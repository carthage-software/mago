<?php

declare(strict_types=1);

/**
 * @return array{state?: string}
 */
function getShape(): array
{
    return [];
}

$address = getShape();

if (null !== ($address['state'] ?? null)) {
    $state = $address['state'];
}
