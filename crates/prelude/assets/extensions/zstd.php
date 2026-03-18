<?php

namespace {
    const ZSTD_COMPRESS_LEVEL_MIN = 1;

    const ZSTD_COMPRESS_LEVEL_MAX = 22;

    const ZSTD_COMPRESS_LEVEL_DEFAULT = 3;

    const ZSTD_VERSION_NUMBER = 10405;

    const ZSTD_VERSION_TEXT = '1.4.5';

    function zstd_compress(
        string $data,
        int $level = ZSTD_COMPRESS_LEVEL_DEFAULT,
        ?string $dict = null,
    ): string|false {}

    function zstd_uncompress(string $data, ?string $dict = null): string|false {}

    /** @deprecated */
    function zstd_compress_dict(string $data, string $dict, int $level = ZSTD_COMPRESS_LEVEL_DEFAULT): string|false {}

    /** @deprecated */
    function zstd_uncompress_dict(string $data, string $dict): string|false {}

    function zstd_compress_init(
        int $level = ZSTD_COMPRESS_LEVEL_DEFAULT,
        ?string $dict = null,
    ): Zstd\Compress\Context|false {}

    function zstd_compress_add(Zstd\Compress\Context $context, string $data, bool $end = false): string|false {}

    function zstd_uncompress_init(?string $dict = null): Zstd\UnCompress\Context|false {}

    function zstd_uncompress_add(Zstd\UnCompress\Context $context, string $data): string|false {}
}

namespace Zstd {
    function compress(string $data, int $level = ZSTD_COMPRESS_LEVEL_DEFAULT, ?string $dict = null): string|false {}

    function uncompress(string $data, ?string $dict = null): string|false {}

    /** @deprecated */
    function compress_dict(string $data, string $dict, int $level = ZSTD_COMPRESS_LEVEL_DEFAULT): string|false {}

    /** @deprecated */
    function uncompress_dict(string $data, string $dict): string|false {}

    function compress_init(int $level = ZSTD_COMPRESS_LEVEL_DEFAULT, ?string $dict = null): Compress\Context|false {}

    function compress_add(Compress\Context $context, string $data, bool $end = false): string|false {}

    function uncompress_init(?string $dict = null): UnCompress\Context|false {}

    function uncompress_add(UnCompress\Context $context, string $data): string|false {}
}

namespace Zstd\Compress {
    final class Context {}
}

namespace Zstd\UnCompress {
    final class Context {}
}
