<?php

class ZipArchive implements Countable
{
    public const LIBZIP_VERSION = '1.7.3';

    public const CREATE = 1;

    public const EXCL = 2;

    public const CHECKCONS = 4;

    public const OVERWRITE = 8;

    public const FL_NOCASE = 1;

    public const FL_NODIR = 2;

    public const FL_COMPRESSED = 4;

    public const FL_UNCHANGED = 8;

    public const FL_RECOMPRESS = 16;

    public const FL_ENCRYPTED = 32;

    public const FL_OVERWRITE = 8192;

    public const FL_LOCAL = 256;

    public const FL_CENTRAL = 512;

    public const EM_TRAD_PKWARE = 1;

    public const EM_UNKNOWN = 65535;

    public const CM_DEFAULT = -1;

    public const CM_STORE = 0;

    public const CM_SHRINK = 1;

    public const CM_REDUCE_1 = 2;

    public const CM_REDUCE_2 = 3;

    public const CM_REDUCE_3 = 4;

    public const CM_REDUCE_4 = 5;

    public const CM_IMPLODE = 6;

    public const CM_DEFLATE = 8;

    public const CM_DEFLATE64 = 9;

    public const CM_PKWARE_IMPLODE = 10;

    public const CM_BZIP2 = 12;

    public const CM_LZMA = 14;

    public const CM_TERSE = 18;

    public const CM_LZ77 = 19;

    public const CM_WAVPACK = 97;

    public const CM_PPMD = 98;

    public const ER_OK = 0;

    public const ER_MULTIDISK = 1;

    public const ER_RENAME = 2;

    public const ER_CLOSE = 3;

    public const ER_SEEK = 4;

    public const ER_READ = 5;

    public const ER_WRITE = 6;

    public const ER_CRC = 7;

    public const ER_ZIPCLOSED = 8;

    public const ER_NOENT = 9;

    public const ER_EXISTS = 10;

    public const ER_OPEN = 11;

    public const ER_TMPOPEN = 12;

    public const ER_ZLIB = 13;

    public const ER_MEMORY = 14;

    public const ER_CHANGED = 15;

    public const ER_COMPNOTSUPP = 16;

    public const ER_EOF = 17;

    public const ER_INVAL = 18;

    public const ER_NOZIP = 19;

    public const ER_INTERNAL = 20;

    public const ER_INCONS = 21;

    public const ER_REMOVE = 22;

    public const ER_DELETED = 23;

    public const EM_NONE = 0;

    public const EM_AES_128 = 257;

    public const EM_AES_192 = 258;

    public const EM_AES_256 = 259;

    public const RDONLY = 16;

    public const FL_ENC_GUESS = 0;

    public const FL_ENC_RAW = 64;

    public const FL_ENC_STRICT = 128;

    public const FL_ENC_UTF_8 = 2048;

    public const FL_ENC_CP437 = 4096;

    public const CM_LZMA2 = 33;

    public const CM_XZ = 95;

    public const ER_ENCRNOTSUPP = 24;

    public const ER_RDONLY = 25;

    public const ER_NOPASSWD = 26;

    public const ER_WRONGPASSWD = 27;

    public const ER_OPNOTSUPP = 28;

    public const ER_INUSE = 29;

    public const ER_TELL = 30;

    public const ER_COMPRESSED_DATA = 31;

    public const ER_CANCELLED = 32;

    public const OPSYS_DOS = 0;

    public const OPSYS_AMIGA = 1;

    public const OPSYS_OPENVMS = 2;

    public const OPSYS_UNIX = 3;

    public const OPSYS_VM_CMS = 4;

    public const OPSYS_ATARI_ST = 5;

    public const OPSYS_OS_2 = 6;

    public const OPSYS_MACINTOSH = 7;

    public const OPSYS_Z_SYSTEM = 8;

    public const OPSYS_WINDOWS_NTFS = 10;

    public const OPSYS_MVS = 11;

    public const OPSYS_VSE = 12;

    public const OPSYS_ACORN_RISC = 13;

    public const OPSYS_VFAT = 14;

    public const OPSYS_ALTERNATE_MVS = 15;

    public const OPSYS_BEOS = 16;

    public const OPSYS_TANDEM = 17;

    public const OPSYS_OS_400 = 18;

    public const OPSYS_OS_X = 19;

    public const OPSYS_CPM = 9;

    public const OPSYS_DEFAULT = 3;

    public const FL_OPEN_FILE_NOW = 1073741824;

    public const CM_ZSTD = 93;

    public const ER_DATA_LENGTH = 33;

    public const ER_NOT_ALLOWED = 34;

    public const AFL_RDONLY = 2;

    public const AFL_IS_TORRENTZIP = 4;

    public const AFL_WANT_TORRENTZIP = 8;

    public const AFL_CREATE_OR_KEEP_FILE_FOR_EMPTY_ARCHIVE = 16;

    public const LENGTH_TO_END = 0;

    public const LENGTH_UNCHECKED = -2;

    public int $status;

    public int $statusSys;

    public int $numFiles;

    public string $filename;

    public string $comment;

    public int $lastId;

    public function open(string $filename, int $flags = null): int|bool
    {
    }

    public function close(): bool
    {
    }

    public function count(): int
    {
    }

    public function getStatusString(): string
    {
    }

    public function addEmptyDir(string $dirname, int $flags): bool
    {
    }

    public function addFromString(string $name, string $content, int $flags = 8192): bool
    {
    }

    public function addFile(
        string $filepath,
        string $entryname = null,
        int $start = 0,
        int $length = 0,
        int $flags = 8192,
    ): bool {
    }

    public function addGlob(string $pattern, int $flags = 0, array $options = []): array|false
    {
    }

    public function addPattern(string $pattern, string $path = '.', array $options = []): array|false
    {
    }

    public function renameIndex(int $index, string $new_name): bool
    {
    }

    public function renameName(string $name, string $new_name): bool
    {
    }

    public function setArchiveComment(string $comment): bool
    {
    }

    public function getArchiveComment(int $flags = null): string|false
    {
    }

    public function setCommentIndex(int $index, string $comment): bool
    {
    }

    public function setCommentName(string $name, string $comment): bool
    {
    }

    public function setCompressionIndex(int $index, int $method, int $compflags = 0): bool
    {
    }

    public function setCompressionName(string $name, int $method, int $compflags = 0): bool
    {
    }

    public function setEncryptionIndex(int $index, int $method, null|string $password = null): bool
    {
    }

    public function setEncryptionName(string $name, int $method, null|string $password = null): bool
    {
    }

    public function setPassword(string $password): bool
    {
    }

    public function getCommentIndex(int $index, int $flags = null): string|false
    {
    }

    public function getCommentName(string $name, int $flags = null): string|false
    {
    }

    public function deleteIndex(int $index): bool
    {
    }

    public function deleteName(string $name): bool
    {
    }

    /**
     * @return array{name: string, index: int, crc: int, size: int, mtime: int, comp_size: int, comp_method: int, encryption_method: int}|false
     */
    public function statName(string $name, int $flags = null): array|false
    {
    }

    public function statIndex(int $index, int $flags = null): array|false
    {
    }

    public function locateName(string $name, int $flags = null): int|false
    {
    }

    public function getNameIndex(int $index, int $flags = null): string|false
    {
    }

    public function unchangeArchive(): bool
    {
    }

    public function unchangeAll(): bool
    {
    }

    public function unchangeIndex(int $index): bool
    {
    }

    public function unchangeName(string $name): bool
    {
    }

    /**
     * @param array<string>|string|null $files
     */
    public function extractTo(string $pathto, array|string|null $files = null): bool
    {
    }

    public function getFromName(string $name, int $len = 0, int $flags = null): string|false
    {
    }

    public function getFromIndex(int $index, int $len = 0, int $flags = null): string|false
    {
    }

    /**
     * @return resource|false
     */
    public function getStream(string $name)
    {
    }

    /**
     * @return resource|false
     */
    public function getStreamIndex(int $index, int $flags = 0)
    {
    }

    public function setExternalAttributesName(string $name, int $opsys, int $attr, int $flags = null): bool
    {
    }

    public function getExternalAttributesName(string $name, int &$opsys, int &$attr, int $flags = null): bool
    {
    }

    public function setExternalAttributesIndex(int $index, int $opsys, int $attr, int $flags = null): bool
    {
    }

    public function getExternalAttributesIndex(int $index, int &$opsys, int &$attr, int $flags = null): bool
    {
    }

    public static function isEncryptionMethodSupported(int $method, bool $enc = true): bool
    {
    }

    public static function isCompressionMethodSupported(int $method, bool $enc = true): bool
    {
    }

    public function registerCancelCallback(callable $callback): bool
    {
    }

    public function registerProgressCallback(float $rate, callable $callback): bool
    {
    }

    public function setMtimeName(string $name, int $timestamp, int $flags = null): bool
    {
    }

    public function setMtimeIndex(int $index, int $timestamp, int $flags = null): bool
    {
    }

    public function replaceFile(
        string $filepath,
        int $index,
        int $start = null,
        int $length = null,
        int $flags = null,
    ): bool {
    }

    public function clearError(): void
    {
    }

    public function setArchiveFlag(int $flag, int $value): bool
    {
    }

    public function getArchiveFlag(int $flag, int $flags = 0): int
    {
    }

    public function getStreamName(string $name, int $flags = 0)
    {
    }
}

/**
 * @return resource|int|false
 */
function zip_open(string $filename)
{
}

/**
 * @param resource $zip
 */
function zip_close($zip): void
{
}

/**
 * @param resource $zip
 *
 * @return resource|false
 *
 * @deprecated
 */
function zip_read($zip)
{
}

/**
 * @param resource $zip_dp
 * @param resource $zip_entry
 *
 * @deprecated
 */
function zip_entry_open($zip_dp, $zip_entry, string $mode = 'rb'): bool
{
}

/**
 * @param resource $zip_entry
 */
function zip_entry_close($zip_entry): bool
{
}

/**
 * @param resource $zip_entry
 */
function zip_entry_read($zip_entry, int $len = 1024): string|false
{
}

/**
 * @param resource $zip_entry
 */
function zip_entry_filesize($zip_entry): int|false
{
}

/**
 * @param resource $zip_entry
 */
function zip_entry_name($zip_entry): string|false
{
}

/**
 * @param resource $zip_entry
 */
function zip_entry_compressedsize($zip_entry): int|false
{
}

/**
 * @param resource $zip_entry
 */
function zip_entry_compressionmethod($zip_entry): string|false
{
}
