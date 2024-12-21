<?php

/**
 * Stubs for Brotli extension (https://pecl.php.net/package/brotli, https://github.com/kjdev/php-ext-brotli)
 * General documentation on the Brotlie algorythm can be found on https://github.com/google/brotli/
 */

namespace {
    /**
     * Minimal brotli compression level
     */
    define('BROTLI_COMPRESS_LEVEL_MIN', 0);
    /**
     * Maximum brotli compression level
     */
    define('BROTLI_COMPRESS_LEVEL_MAX', 11);
    /**
     * Default brotli compression level (internally resolves to 11)
     */
    define('BROTLI_COMPRESS_LEVEL_DEFAULT', -1);
    /**
     * Default brotli compression mode. In this mode compressor does not know anything in advance about the properties of the input.
     */
    define('BROTLI_GENERIC', 0);
    /**
     * Brotli compression mode for UTF-8 formatted text input.
     */
    define('BROTLI_TEXT', 1);
    /**
     * Brotli compression mode used in WOFF 2.0.
     */
    define('BROTLI_FONT', 2);
    /**
     * Operation to produce output for all processed input.
     * Actual flush is performed when input stream is depleted and there is enough space in output stream.
     * This means that client should repeat the operation until stream is fully processed.
     * When flush is complete, output data will be sufficient for decoder to reproduce all the given input.
     */
    define('BROTLI_FLUSH', 1);
    /**
     * Operation to process data input.
     * Encoder may postpone producing output, until it has processed enough input.
     */
    define('BROTLI_PROCESS', 0);
    /**
     * Operation to finalize the brotli stream.
     * Actual finalization is performed when input stream is depleted and there is enough space in output stream.
     * Adding more input data to finalized stream is impossible.
     */
    define('BROTLI_FINISH', 2);

    /**
     * This function compress a string using brotli algorithm.
     *
     * @param string $data    The data to compress.
     * @param int    $quality The higher the quality, the slower the compression. Defaults to BROTLI_COMPRESS_LEVEL_DEFAULT.
     * @param int    $mode    The compression mode can be BROTLI_GENERIC (default), BROTLI_TEXT (for UTF-8 format text input) or BROTLI_FONT (for WOFF 2.0).
     *
     * @return string|false The compressed string or FALSE on error.
     */
    function brotli_compress(string $data, int $quality = BROTLI_COMPRESS_LEVEL_DEFAULT, int $mode = BROTLI_GENERIC): string|false {}

    /**
     * This function uncompresses a string compressed with brotli algorithm.
     *
     * @param string $data   The data to uncompress.
     * @param int    $length The higher the quality, the slower the compression.
     *
     * @return string|false The uncompressed string or FALSE on error.
     */
    function brotli_uncompress(string $data, int $length = 0): string|false {}

    /**
     * Initialize an incremental brotli compress context.
     *
     * @param int $quality The higher the quality, the slower the compression. Defaults to BROTLI_COMPRESS_LEVEL_DEFAULT.
     * @param int $mode    The compression mode can be BROTLI_GENERIC (default), BROTLI_TEXT (for UTF-8 format text input) or BROTLI_FONT (for WOFF 2.0).
     *
     * @return resource|false TReturns a brotli context resource (brotli.state) on success or FALSE on error.
     */
    function brotli_compress_init(int $quality = BROTLI_COMPRESS_LEVEL_DEFAULT, int $mode = BROTLI_GENERIC) {}

    /**
     * Initialize an incremental brotli compress context.
     *
     * @param resource $context A context created with brotli_compress_init()
     * @param string   $data    The data to compress.
     * @param int      $mode    One of BROTLI_FLUSH (default), BROTLI_PROCESS or BROTLI_FINISH.
     *
     * @return string|false TReturns a brotli context resource (brotli.state) on success or FALSE on error.
     */
    function brotli_compress_add($context, string $data, int $mode = BROTLI_FLUSH): string|false {}

    /**
     * Initialize an incremental brotli uncompress context.
     *
     * @return resource|false TReturns a brotli context resource (brotli.state) on success or FALSE on error.
     */
    function brotli_uncompress_init() {}

    /**
     * Initialize an incremental brotli compress context.
     *
     * @param resource $context A context created with brotli_uncompress_init()
     * @param string   $data    The data to uncompress.
     * @param int      $mode    One of BROTLI_FLUSH (default), BROTLI_PROCESS or BROTLI_FINISH.
     *
     * @return string|false TReturns a brotli context resource (brotli.state) on success or FALSE on error.
     */
    function brotli_uncompress_add($context, string $data, int $mode = BROTLI_FLUSH): string|false {}
}

/**
 * Aliases for the functions
 */

namespace Brotli {
    /**
     * This function compress a string using brotli algorithm.
     *
     * @param string $data    The data to compress.
     * @param int    $quality The higher the quality, the slower the compression. Defaults to BROTLI_COMPRESS_LEVEL_DEFAULT.
     * @param int    $mode    The compression mode can be BROTLI_GENERIC (default), BROTLI_TEXT (for UTF-8 format text input) or BROTLI_FONT (for WOFF 2.0).
     *
     * @return string|false The compressed string or FALSE on error.
     */
    function compress(string $data, int $quality = BROTLI_COMPRESS_LEVEL_DEFAULT, int $mode = BROTLI_GENERIC): string|false {}

    /**
     * This function uncompresses a string compressed with brotli algorithm.
     *
     * @param string $data   The data to uncompress.
     * @param int    $length The higher the quality, the slower the compression.
     *
     * @return string|false The uncompressed string or FALSE on error.
     */
    function uncompress(string $data, int $length = 0): string|false {}

    /**
     * Initialize an incremental brotli compress context.
     *
     * @param int $quality The higher the quality, the slower the compression. Defaults to BROTLI_COMPRESS_LEVEL_DEFAULT.
     * @param int $mode    The compression mode can be BROTLI_GENERIC (default), BROTLI_TEXT (for UTF-8 format text input) or BROTLI_FONT (for WOFF 2.0).
     *
     * @return resource|false TReturns a brotli context resource (brotli.state) on success or FALSE on error.
     */
    function compress_init(int $quality = BROTLI_COMPRESS_LEVEL_DEFAULT, int $mode = BROTLI_GENERIC) {}

    /**
     * Initialize an incremental brotli compress context.
     *
     * @param resource $context A context created with brotli_compress_init()
     * @param string   $data    The data to compress.
     * @param int      $mode    One of BROTLI_FLUSH (default), BROTLI_PROCESS or BROTLI_FINISH.
     *
     * @return string|false Returns a brotli context resource (brotli.state) on success or FALSE on error.
     */
    function compress_add($context, string $data, int $mode = BROTLI_FLUSH): string|false {}

    /**
     * Initialize an incremental brotli uncompress context.
     *
     * @return resource|false TReturns a brotli context resource (brotli.state) on success or FALSE on error.
     */
    function uncompress_init() {}

    /**
     * Initialize an incremental brotli compress context.
     *
     * @param resource $context A context created with brotli_uncompress_init()
     * @param string   $data    The data to uncompress.
     * @param int      $mode    One of BROTLI_FLUSH (default), BROTLI_PROCESS or BROTLI_FINISH.
     *
     * @return string|false TReturns a brotli context resource (brotli.state) on success or FALSE on error.
     */
    function uncompress_add($context, string $data, int $mode = BROTLI_FLUSH): string|false {}
}
