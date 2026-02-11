<?php

namespace {
    const ZSTD_COMPRESS_LEVEL_MIN = 1;

    const ZSTD_COMPRESS_LEVEL_MAX = 22;

    const ZSTD_COMPRESS_LEVEL_DEFAULT = 3;

    const LIBZSTD_VERSION_NUMBER = 10405;

    const LIBZSTD_VERSION_STRING = '1.4.5';

    function zstd_compress(string $data, int $level = 3): string|false {}

    function zstd_uncompress(string $data): string|false {}

    function zstd_compress_dict(string $data, string $dict): string|false {}

    function zstd_compress_usingcdict(string $data, string $dict): string|false {}

    function zstd_uncompress_dict(string $data, string $dict): string|false {}

    function zstd_decompress_dict(string $data, string $dict): string|false {}

    function zstd_uncompress_usingcdict(string $data, string $dict): string|false {}

    function zstd_decompress_usingcdict(string $data, string $dict): string|false {}
}

namespace Zstd {
    function compress(string $data, int $level = 3): string|false {}

    function uncompress(string $data): string|false {}

    function compress_dict(string $data, string $dict): string|false {}

    function uncompress_dict(string $data, string $dict): string|false {}
}
