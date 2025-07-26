<?php

class finfo
{
    public function __construct(int $flags = 0, null|string $magic_database = null) {}

    public function set_flags(int $flags): bool
    {
    }

    /**
     * @param null|resource $context
     *
     * @pure
     */
    public function file(string $filename, int $flags = FILEINFO_NONE, mixed $context = null): string|false
    {
    }

    /**
     * @param null|resource $context
     *
     * @pure
     */
    public function buffer(string $string, int $flags = FILEINFO_NONE, mixed $context = null): string|false
    {
    }
}

function finfo_open(int $flags = 0, null|string $magic_database = null): finfo|false
{
}

function finfo_close(finfo $finfo): bool
{
}

function finfo_set_flags(finfo $finfo, int $flags): bool
{
}

/**
 * @param finfo $finfo
 * @param null|resource $context
 */
function finfo_file(finfo $finfo, string $filename, int $flags = 0, mixed $context = null): string|false
{
}

/**
 * @param finfo $finfo
 * @param null|resource $context
 */
function finfo_buffer(finfo $finfo, string $string, int $flags = FILEINFO_NONE, mixed $context = null): string|false
{
}

/**
 * @param resource|object|string $filename
 */
function mime_content_type($filename): string|false
{
}

const FILEINFO_NONE = 0;

const FILEINFO_SYMLINK = 2;

const FILEINFO_MIME = 1040;

const FILEINFO_MIME_TYPE = 16;

const FILEINFO_MIME_ENCODING = 1024;

const FILEINFO_DEVICES = 8;

const FILEINFO_CONTINUE = 32;

const FILEINFO_PRESERVE_ATIME = 128;

const FILEINFO_RAW = 256;

const FILEINFO_EXTENSION = 2097152;

const FILEINFO_APPLE = 2048;
