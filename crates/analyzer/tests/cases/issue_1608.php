<?php

declare(strict_types=1);

function obscure_func(int &$user_id): bool
{
    $user_id = mt_rand(0, max: 10);
    return (bool) mt_rand(0, max: 1);
}

function test(): void
{
    $user_id = mt_rand(0, max: 1);
    if ($user_id === 0 && obscure_func($user_id) && $user_id !== 0) {
        echo "can now show $user_id as owner.\n";
    }
}
