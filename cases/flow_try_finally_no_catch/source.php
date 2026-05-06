<?php

declare(strict_types=1);

function flow_try_finally_no_catch(): void
{
    $resource = 'open';
    try {
        echo $resource;
    } finally {
        $resource = 'closed';
        echo $resource;
    }
}
