<?php

declare(strict_types=1);

/**
 * @param resource $stream
 */
function stream_meta_data_optional_keys($stream): string
{
    $meta = stream_get_meta_data($stream);
    $uri = $meta['uri'] ?? '';
    $timedOut = $meta['timed_out'] ?? false;
    $base64 = isset($meta['base64']);

    return $uri . ($timedOut ? 'y' : 'n') . ($base64 ? 'y' : 'n') . $meta['mode'];
}
