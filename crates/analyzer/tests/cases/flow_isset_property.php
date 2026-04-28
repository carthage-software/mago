<?php

declare(strict_types=1);

final class Profile
{
    public null|string $bio = null;
}

function flow_isset_property(Profile $p): string
{
    if (isset($p->bio)) {
        return $p->bio;
    }

    return '';
}
