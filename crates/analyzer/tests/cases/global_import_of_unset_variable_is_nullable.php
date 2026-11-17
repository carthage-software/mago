<?php

declare(strict_types=1);

function global_response_headers(): int
{
    global $http_response_header;

    /** @mago-expect analysis:possibly-null-argument */
    return count($http_response_header);
}
