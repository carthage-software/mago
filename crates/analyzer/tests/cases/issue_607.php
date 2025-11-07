<?php

declare(strict_types=1);

/**
 * @param array<string, mixed> $payload The complete webhook payload including the Signature field
 */
function get_signature(array $payload): null|string
{
    if (!isset($payload['Signature']) || !is_string($payload['Signature'])) {
        return null;
    }

    return $payload['Signature'];
}
