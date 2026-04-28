<?php

declare(strict_types=1);

function bad_bool_index(bool $b): mixed
{
    // @mago-expect analysis:invalid-array-access
    return $b[0];
}
