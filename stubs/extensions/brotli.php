<?php

namespace {
    const BROTLI_COMPRESS_LEVEL_MIN = 0;

    const BROTLI_COMPRESS_LEVEL_MAX = 11;

    const BROTLI_COMPRESS_LEVEL_DEFAULT = -1;

    const BROTLI_GENERIC = 0;

    const BROTLI_TEXT = 1;

    const BROTLI_FONT = 2;

    const BROTLI_FLUSH = 1;

    const BROTLI_PROCESS = 0;

    const BROTLI_FINISH = 2;

    function brotli_compress(
        string $data,
        int $quality = BROTLI_COMPRESS_LEVEL_DEFAULT,
        int $mode = BROTLI_GENERIC,
    ): string|false {
    }

    function brotli_uncompress(string $data, int $length = 0): string|false
    {
    }

    /**
     * @return resource|false
     */
    function brotli_compress_init(int $quality = BROTLI_COMPRESS_LEVEL_DEFAULT, int $mode = BROTLI_GENERIC)
    {
    }

    /**
     * @param resource $context
     */
    function brotli_compress_add($context, string $data, int $mode = BROTLI_FLUSH): string|false
    {
    }

    /**
     * @return resource|false
     */
    function brotli_uncompress_init()
    {
    }

    /**
     * @param resource $context
     */
    function brotli_uncompress_add($context, string $data, int $mode = BROTLI_FLUSH): string|false
    {
    }
}

namespace Brotli {
    function compress(
        string $data,
        int $quality = BROTLI_COMPRESS_LEVEL_DEFAULT,
        int $mode = BROTLI_GENERIC,
    ): string|false {
    }

    function uncompress(string $data, int $length = 0): string|false
    {
    }

    /**
     * @return resource|false
     */
    function compress_init(int $quality = BROTLI_COMPRESS_LEVEL_DEFAULT, int $mode = BROTLI_GENERIC)
    {
    }

    /**
     * @param resource $context
     */
    function compress_add($context, string $data, int $mode = BROTLI_FLUSH): string|false
    {
    }

    /**
     * @return resource|false
     */
    function uncompress_init()
    {
    }

    /**
     * @param resource $context
     */
    function uncompress_add($context, string $data, int $mode = BROTLI_FLUSH): string|false
    {
    }
}
