<?php

declare(strict_types=1);

class User
{
    public function isAuthorized(): bool
    {
        return true;
    }
}

function test_nullsafe(?User $user): bool
{
    if ($user?->isAuthorized()) {
        return true;
    }

    if (null !== $user) {
        echo 'user is not null!';
    }

    return false;
}

test_nullsafe(new User());
test_nullsafe(null);
