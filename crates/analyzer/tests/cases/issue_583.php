<?php

declare(strict_types=1);

/**
 * @throws InvalidArgumentException
 * @param array{currency: null|string} $data
 */
function x($data): void
{
    $data['currency'] ?? throw new \InvalidArgumentException('Currency is required.');
    accept_string($data['currency']);
}

function accept_string(string $_s): void
{
}
