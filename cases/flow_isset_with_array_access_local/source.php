<?php

declare(strict_types=1);

function flow_isset_with_array_access_local(): string
{
    $arr = ['name' => 'alice'];

    if (isset($arr['name'])) {
        return $arr['name'];
    }

    return '';
}
