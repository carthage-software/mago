<?php

declare(strict_types=1);

/**
 * @param array{email?: string, password?: string} $user
 */
function validateUser(array $user): bool
{
    if (!isset($user['email']) || !isset($user['password'])) {
        return false;
    }

    if ($user['email'] === '' || $user['password'] === '') {
        return false;
    }

    return true;
}

/**
 * @param array<string, int>|null $data
 */
function processOptionalData(null|array $data, bool $strict): void
{
    if ($data === null || !$strict) {
        return;
    }

    if (empty($data)) {
        return;
    }

    foreach ($data as $key => $value) {
        echo "{$key} => {$value}\n";
    }
}

function test(): void
{
    var_dump(validateUser(['email' => 'test@example.com', 'password' => 'secret']));
    var_dump(validateUser(['email' => '', 'password' => 'secret']));

    processOptionalData(['a' => 1, 'b' => 2], true);
}
