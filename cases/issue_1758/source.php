<?php

declare(strict_types=1);

function read_with_path_only(): void
{
    $e = exif_read_data('/tmp/photo.jpg');
    print_r($e);
}

function read_with_resource(): void
{
    $handle = fopen('/tmp/photo.jpg', 'rb');
    if ($handle === false) {
        return;
    }

    $e = exif_read_data($handle);
    print_r($e);
}
