<?php

/**
 * @param string $file
 */
function exif_read_data(
    $file,
    ?string $required_sections,
    bool $as_arrays = false,
    bool $read_thumbnail = false,
): array|false {}

function exif_tagname(int $index): string|false {}

/**
 * @param string|resource $file
 * @param int $width
 * @param int $height
 * @param int $image_type
 */
function exif_thumbnail($file, &$width, &$height, &$image_type): string|false {}

function exif_imagetype(string $filename): int|false {}

const EXIF_USE_MBSTRING = 1;
