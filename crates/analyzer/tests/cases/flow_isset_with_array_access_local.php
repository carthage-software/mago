<?php

declare(strict_types=1);

function flow_isset_with_array_access_local(): string
{
    $arr = ['name' => 'alice'];

    /** @mago-expect analysis:redundant-condition */
    if (isset($arr['name'])) {
        return $arr['name'];
    }

    return '';
}
