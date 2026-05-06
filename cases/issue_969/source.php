<?php

/**
 * @param string $file
 *
 * @return array{basename: string, dirname?: string, extension?: string, filename: string}
 */
function get_file_information(string $file): array
{
    return pathinfo($file, PATHINFO_ALL);
}

/**
 * @param string $file
 *
 * @return array{basename: string, dirname?: string, extension?: string, filename: string}
 */
function get_file_information2(string $file): array
{
    return pathinfo($file);
}
