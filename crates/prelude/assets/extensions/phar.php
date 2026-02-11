<?php

class PharException extends Exception {}

class Phar extends RecursiveDirectoryIterator implements RecursiveIterator, SeekableIterator, Countable, ArrayAccess
{
    public const BZ2 = 8192;
    public const GZ = 4096;
    public const NONE = 0;
    public const PHAR = 1;
    public const TAR = 2;
    public const ZIP = 3;
    public const COMPRESSED = 61440;
    public const PHP = 0;
    public const PHPS = 1;
    public const MD5 = 1;
    public const OPENSSL = 16;
    public const SHA1 = 2;
    public const SHA256 = 3;
    public const SHA512 = 4;
    public const OPENSSL_SHA256 = 5;
    public const OPENSSL_SHA512 = 6;

    /**
     * @throws BadMethodCallException
     * @throws UnexpectedValueException
     */
    public function __construct(
        string $filename,
        int $flags = FilesystemIterator::SKIP_DOTS | FilesystemIterator::UNIX_PATHS,
        ?string $alias = null,
    ) {}

    public function __destruct() {}

    public function addEmptyDir(string $directory): void {}

    public function addFile(string $filename, ?string $localName = null): void {}

    public function addFromString(string $localName, string $contents): void {}

    public function buildFromDirectory(string $directory, string $pattern = ''): array {}

    public function buildFromIterator(Traversable $iterator, ?string $baseDirectory = null): array {}

    public function compressFiles(int $compression): void {}

    public function decompressFiles(): bool {}

    public function compress(int $compression, ?string $extension = null): ?Phar {}

    public function decompress(?string $extension = null): ?Phar {}

    public function convertToExecutable(
        ?int $format = null,
        ?int $compression = null,
        ?string $extension = null,
    ): ?Phar {}

    public function convertToData(
        ?int $format = null,
        ?int $compression = null,
        ?string $extension = null,
    ): ?PharData {}

    public function copy(string $to, string $from): bool {}

    /**
     * @return int<0, max>
     */
    public function count(int $mode = COUNT_NORMAL): int {}

    public function delete(string $localName): bool {}

    public function delMetadata(): bool {}

    public function extractTo(string $directory, array|string|null $files = null, bool $overwrite = false): bool {}

    public function getAlias(): ?string {}

    public function getMetadata(array $unserializeOptions = []): mixed {}

    public function getModified(): bool {}

    /**
     * @return false|array{
     *   hash: string,
     *   hash_type: string
     * }
     */
    public function getSignature(): array|false {}

    public function getStub(): string {}

    public function getVersion(): string {}

    public function hasMetadata(): bool {}

    public function isBuffering(): bool {}

    public function isCompressed(): int|false {}

    public function isFileFormat(int $format): bool {}

    public function isWritable(): bool {}

    /**
     * @param string $localName
     */
    public function offsetExists($localName): bool {}

    /**
     * @param string $localName
     */
    public function offsetGet($localName): SplFileInfo {}

    /**
     * @param string $localName
     * @param string $value
     */
    public function offsetSet($localName, $value): void {}

    /**
     * @param string $localName
     */
    public function offsetUnset($localName): void {}

    public function setAlias(string $alias): bool {}

    public function setDefaultStub(?string $index = null, ?string $webIndex = null): bool {}

    public function setMetadata(mixed $metadata): void {}

    public function setSignatureAlgorithm(int $algo, ?string $privateKey = null): void {}

    /**
     * @param string $stub
     */
    public function setStub($stub, int $length): bool {}

    public function startBuffering(): void {}

    public function stopBuffering(): void {}

    final public static function apiVersion(): string {}

    final public static function canCompress(int $compression = 0): bool {}

    final public static function canWrite(): bool {}

    final public static function createDefaultStub(?string $index = null, ?string $webIndex = null): string {}

    /**
     * @return list<non-empty-string>
     */
    final public static function getSupportedCompression(): array {}

    /**
     * @return list<non-empty-string>
     */
    final public static function getSupportedSignatures(): array {}

    final public static function interceptFileFuncs(): void {}

    final public static function isValidPharFilename(string $filename, bool $executable = true): bool {}

    final public static function loadPhar(string $filename, ?string $alias = null): bool {}

    final public static function mapPhar(?string $alias = null, int $offset = 0): bool {}

    final public static function running(bool $returnPhar = true): string {}

    final public static function mount(string $pharPath, string $externalPath): void {}

    final public static function mungServer(array $variables): void {}

    /**
     * @throws PharException
     */
    final public static function unlinkArchive(string $filename): bool {}

    final public static function webPhar(
        ?string $alias = null,
        ?string $index = null,
        ?string $fileNotFoundScript = null,
        array $mimeTypes = [],
        ?callable $rewrite = null,
    ): void {}

    /**
     * @param bool $allow_links
     *
     * @return bool
     */
    public function hasChildren($allow_links = false) {}

    public function getChildren() {}

    /**
     * @return void
     */
    public function rewind() {}

    /**
     * @return void
     */
    public function next() {}

    /**
     * @return string
     */
    public function key() {}

    public function current() {}

    /**
     * @return bool
     */
    public function valid() {}

    /**
     * @param int $position
     *
     * @return void
     */
    public function seek($position) {}

    public function _bad_state_ex() {}
}

class PharData extends Phar
{
    public function __construct(
        string $filename,
        int $flags = FilesystemIterator::SKIP_DOTS | FilesystemIterator::UNIX_PATHS,
        ?string $alias = null,
        int $format = 0,
    ) {}

    /**
     * @param string $localName
     */
    public function offsetExists($localName): bool {}

    /**
     * @param string $localName
     */
    public function offsetGet($localName): SplFileInfo {}

    /**
     * @param string $localName
     * @param string $value
     */
    public function offsetSet($localName, $value): void {}

    /**
     * @param string $localName
     */
    public function offsetUnset($localName): void {}

    /**
     * @param bool $allow_links
     *
     * @return bool
     */
    public function hasChildren($allow_links = false) {}

    public function getChildren() {}

    public function rewind() {}

    public function next() {}

    /**
     * @return string
     */
    public function key() {}

    public function current() {}

    /**
     * @return bool
     */
    public function valid() {}

    /**
     * @param int $position
     *
     * @return void
     */
    public function seek($position) {}
}

class PharFileInfo extends SplFileInfo
{
    public function __construct(string $filename) {}

    public function __destruct() {}

    public function chmod(int $perms): void {}

    public function compress(int $compression): bool {}

    public function decompress(): bool {}

    public function delMetadata(): bool {}

    public function getCompressedSize(): int {}

    public function getCRC32(): int {}

    public function getContent(): string {}

    public function getMetadata(array $unserializeOptions = []): mixed {}

    public function getPharFlags(): int {}

    public function hasMetadata(): bool {}

    public function isCompressed(?int $compression = null): bool {}

    public function isCRCChecked(): bool {}

    public function setMetadata(mixed $metadata): void {}
}
