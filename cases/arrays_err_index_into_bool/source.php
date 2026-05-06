<?php

declare(strict_types=1);

function bad_bool_index(bool $b): mixed
{
    return $b[0];
}
