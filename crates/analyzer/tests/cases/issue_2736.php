<?php

declare(strict_types=1);

function take_handle(CurlHandle $handle): void
{
    curl_multi_getcontent($handle);
}

function read_messages(CurlMultiHandle $multi): int
{
    $queued = 0;
    $info = curl_multi_info_read($multi, $queued);
    if ($info !== false) {
        take_handle($info['handle']);
    }

    return $queued;
}
