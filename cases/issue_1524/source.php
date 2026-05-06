<?php

declare(strict_types=1);

function usr_auth_is_limited(int $uid, ?bool &$approved = null): bool
{
    $approved = true;
    return false;
}

$uid = 77;
if ($uid) {
    $approved = false;
    if (usr_auth_is_limited($uid, $approved) && !$approved) {
        echo 'not approved';
    }
    echo 'something else';
}
