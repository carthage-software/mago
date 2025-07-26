<?php

function xdiff_file_bdiff_size(string $file): int
{
}

function xdiff_file_bdiff(string $old_file, string $new_file, string $dest): bool
{
}

function xdiff_file_bpatch(string $file, string $patch, string $dest): bool
{
}

function xdiff_file_diff_binary(string $old_file, string $new_file, string $dest): bool
{
}

function xdiff_file_diff(
    string $old_file,
    string $new_file,
    string $dest,
    int $context = 3,
    bool $minimal = false,
): bool {
}

/**@return bool|string
 */
function xdiff_file_merge3(string $old_file, string $new_file1, string $new_file2, string $dest)
{
}

function xdiff_file_patch_binary(string $file, string $patch, string $dest): bool
{
}

/**
 * @return bool|string
 */
function xdiff_file_patch(string $file, string $patch, string $dest, int $flags = XDIFF_PATCH_NORMAL)
{
}

/**
 * @return bool
 */
function xdiff_file_rabdiff(string $old_file, string $new_file, string $dest): bool
{
}

function xdiff_string_bdiff_size(string $patch): int
{
}

/**
 * @return string|false
 */
function xdiff_string_bdiff(string $old_data, string $new_data)
{
}

/**
 * @return string|false
 */
function xdiff_string_bpatch(string $str, string $patch)
{
}

/**
 * @return string|false
 */
function xdiff_string_diff_binary(string $old_data, string $new_data)
{
}

/**
 * @return string|false
 */
function xdiff_string_diff(string $old_data, string $new_data, int $context = 3, bool $minimal = false)
{
}

/**
 * @return bool|string
 */
function xdiff_string_merge3(string $old_data, string $new_data1, string $new_data2, null|string &$error)
{
}

/**
 * @return string|false
 */
function xdiff_string_patch_binary(string $str, string $patch)
{
}

/**
 * @return string|false
 */
function xdiff_string_patch(string $str, string $patch, null|int $flags, null|string &$error)
{
}

/**
 * @return string|false
 */
function xdiff_string_rabdiff(string $old_data, string $new_data)
{
}

const XDIFF_PATCH_NORMAL = 0;

const XDIFF_PATCH_REVERSE = 0;

const XDIFF_PATCH_IGNORESPACE = 0;
