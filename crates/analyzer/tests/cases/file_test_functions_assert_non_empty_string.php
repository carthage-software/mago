<?php

declare(strict_types=1);

/**
 * @param non-empty-string $path
 */
function file_test_takes_non_empty_string(string $path): void
{
    echo $path;
}

function file_test_narrows(string $path): void
{
    if (file_exists($path)) {
        file_test_takes_non_empty_string($path);
    }

    if (is_file($path) || is_dir($path) || is_link($path)) {
        file_test_takes_non_empty_string($path);
    }

    if (is_readable($path) && is_writable($path) && is_writeable($path) && is_executable($path)) {
        file_test_takes_non_empty_string($path);
    }
}

function file_test_on_empty_string(): void
{
    $path = '';
    /** @mago-expect analysis:impossible-type-comparison */
    if (is_file($path)) {
        echo 'never';
    }
}
