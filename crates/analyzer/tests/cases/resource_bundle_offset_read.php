<?php

declare(strict_types=1);

function resource_bundle_offset_read(ResourceBundle $bundle): string
{
    $languages = $bundle['Languages'];
    if (!$languages instanceof ResourceBundle) {
        return '';
    }

    $name = $languages['en'];

    return is_string($name) ? $name : '';
}
