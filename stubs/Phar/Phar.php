<?php

// Start of Phar v.2.0.1

use JetBrains\PhpStorm\ArrayShape;
use JetBrains\PhpStorm\Internal\LanguageLevelTypeAware;
use JetBrains\PhpStorm\Internal\PhpStormStubsElementAvailable;
use JetBrains\PhpStorm\Internal\TentativeType;

/**
 * The PharException class provides a phar-specific exception class
 * for try/catch blocks.
 * @link https://php.net/manual/en/class.pharexception.php
 */
class PharException extends Exception
{
}

/**
 * The Phar class provides a high-level interface to accessing and creating
 * phar archives.
 * @link https://php.net/manual/en/class.phar.php
 */
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
     * (PHP &gt;= 5.3.0, PECL phar &gt;= 1.0.0)<br/>
     * Construct a Phar archive object
     * @link https://php.net/manual/en/phar.construct.php
     * @param string $filename <p>
     * Path to an existing Phar archive or to-be-created archive. The file name's
     * extension must contain .phar.
     * </p>
     * @param int $flags [optional] <p>
     * Flags to pass to parent class <b>RecursiveDirectoryIterator</b>.
     * </p>
     * @param string $alias [optional] <p>
     * Alias with which this Phar archive should be referred to in calls to stream
     * functionality.
     * </p>
     * @throws BadMethodCallException If called twice.
     * @throws UnexpectedValueException If the phar archive can't be opened.
     */
    public function __construct(
        string $filename,
        int $flags = FilesystemIterator::SKIP_DOTS | FilesystemIterator::UNIX_PATHS,
        string|null $alias = null,
        #[PhpStormStubsElementAvailable(from: '5.3', to: '5.6')]  $fileformat = null,
    ) {}

    public function __destruct()
    {
    }

    public function addEmptyDir(string $directory): void
    {
    }

    public function addFile(string $filename, string|null $localName = null): void
    {
    }

    public function addFromString(string $localName, string $contents): void
    {
    }

    public function buildFromDirectory(string $directory, string $pattern = ''): array
    {
    }

    public function buildFromIterator(Traversable $iterator, string|null $baseDirectory = null): array
    {
    }

    public function compressFiles(int $compression): void
    {
    }

    public function decompressFiles(): bool
    {
    }

    public function compress(int $compression, string|null $extension = null): null|Phar
    {
    }

    public function decompress(string|null $extension = null): null|Phar
    {
    }

    /**
     * (PHP &gt;= 5.3.0, PECL phar &gt;= 2.0.0)<br/>
     * Convert a phar archive to another executable phar archive file format
     * @link https://php.net/manual/en/phar.converttoexecutable.php
     * @param int $format [optional] <p>
     * This should be one of Phar::PHAR, Phar::TAR,
     * or Phar::ZIP. If set to <b>NULL</b>, the existing file format
     * will be preserved.
     * </p>
     * @param int $compression [optional] <p>
     * This should be one of Phar::NONE for no whole-archive
     * compression, Phar::GZ for zlib-based compression, and
     * Phar::BZ2 for bzip-based compression.
     * </p>
     * @param string $extension [optional] <p>
     * This parameter is used to override the default file extension for a
     * converted archive. Note that all zip- and tar-based phar archives must contain
     * .phar in their file extension in order to be processed as a
     * phar archive.
     * </p>
     * <p>
     * If converting to a phar-based archive, the default extensions are
     * .phar, .phar.gz, or .phar.bz2
     * depending on the specified compression. For tar-based phar archives, the
     * default extensions are .phar.tar, .phar.tar.gz,
     * and .phar.tar.bz2. For zip-based phar archives, the
     * default extension is .phar.zip.
     * </p>
     * @return Phar|null The method returns a <b>Phar</b> object on success and throws an
     * exception on failure.
     */
    public function convertToExecutable(
        int|null $format = null,
        int|null $compression = null,
        string|null $extension = null,
    ): null|Phar {
    }

    /**
     * (PHP &gt;= 5.3.0, PECL phar &gt;= 2.0.0)<br/>
     * Convert a phar archive to a non-executable tar or zip file
     * @link https://php.net/manual/en/phar.converttodata.php
     * @param int $format [optional] <p>
     * This should be one of Phar::TAR
     * or Phar::ZIP. If set to <b>NULL</b>, the existing file format
     * will be preserved.
     * </p>
     * @param int $compression [optional] <p>
     * This should be one of Phar::NONE for no whole-archive
     * compression, Phar::GZ for zlib-based compression, and
     * Phar::BZ2 for bzip-based compression.
     * </p>
     * @param string $extension [optional] <p>
     * This parameter is used to override the default file extension for a
     * converted archive. Note that .phar cannot be used
     * anywhere in the filename for a non-executable tar or zip archive.
     * </p>
     * <p>
     * If converting to a tar-based phar archive, the
     * default extensions are .tar, .tar.gz,
     * and .tar.bz2 depending on specified compression.
     * For zip-based archives, the
     * default extension is .zip.
     * </p>
     * @return PharData|null The method returns a <b>PharData</b> object on success and throws an
     * exception on failure.
     */
    public function convertToData(
        int|null $format = null,
        int|null $compression = null,
        string|null $extension = null,
    ): null|PharData {
    }

    /**
     * (PHP &gt;= 5.3.0, PECL phar &gt;= 2.0.0)<br/>
     * Copy a file internal to the phar archive to another new file within the phar
     * @link https://php.net/manual/en/phar.copy.php
     * @param string $to
     * @param string $from
     * @return bool returns <b>TRUE</b> on success, but it is safer to encase method call in a
     * try/catch block and assume success if no exception is thrown.
     */
    #[LanguageLevelTypeAware(['8.4' => 'true'], default: 'bool')]
    public function copy(string $to, string $from)
    {
    }

    /**
     * (PHP &gt;= 5.3.0, PECL phar &gt;= 1.0.0)<br/>
     * Returns the number of entries (files) in the Phar archive
     * @link https://php.net/manual/en/phar.count.php
     * @param int $mode [optional]
     * @return int<0,max> The number of files contained within this phar, or 0 (the number zero)
     * if none.
     */
    public function count(#[PhpStormStubsElementAvailable(from: '8.0')] int $mode = COUNT_NORMAL): int
    {
    }

    /**
     * (PHP &gt;= 5.3.0, PECL phar &gt;= 2.0.0)<br/>
     * Delete a file within a phar archive
     * @link https://php.net/manual/en/phar.delete.php
     * @param string $localName <p>
     * Path within an archive to the file to delete.
     * </p>
     * @return bool returns <b>TRUE</b> on success, but it is better to check for thrown exception,
     * and assume success if none is thrown.
     */
    #[LanguageLevelTypeAware(['8.4' => 'true'], default: 'bool')]
    public function delete(string $localName)
    {
    }

    /**
     * (PHP &gt;= 5.3.0, PECL phar &gt;= 1.2.0)<br/>
     * Deletes the global metadata of the phar
     * @link https://php.net/manual/en/phar.delmetadata.php
     * @return bool returns <b>TRUE</b> on success, but it is better to check for thrown exception,
     * and assume success if none is thrown.
     */
    #[LanguageLevelTypeAware(['8.4' => 'true'], default: 'bool')]
    public function delMetadata()
    {
    }

    /**
     * (Unknown)<br/>
     * Extract the contents of a phar archive to a directory
     * @link https://php.net/manual/en/phar.extractto.php
     * @param string $directory <p>
     * Path within an archive to the file to delete.
     * </p>
     * @param string|array|null $files [optional] <p>
     * The name of a file or directory to extract, or an array of files/directories to extract
     * </p>
     * @param bool $overwrite [optional] <p>
     * Set to <b>TRUE</b> to enable overwriting existing files
     * </p>
     * @return bool returns <b>TRUE</b> on success, but it is better to check for thrown exception,
     * and assume success if none is thrown.
     */
    public function extractTo(string $directory, array|string|null $files = null, bool $overwrite = false): bool
    {
    }

    /**
     * @return string|null
     * @see setAlias
     */
    public function getAlias(): null|string
    {
    }

    /**
     * (PHP &gt;= 5.3.0, PECL phar &gt;= 1.0.0)<br/>
     * Returns phar archive meta-data
     * @link https://php.net/manual/en/phar.getmetadata.php
     * @param array $unserializeOptions [optional] if is set to anything other than the default,
     * the resulting metadata won't be cached and this won't return the value from the cache
     * @return mixed any PHP variable that can be serialized and is stored as meta-data for the Phar archive,
     * or <b>NULL</b> if no meta-data is stored.
     */
    public function getMetadata(#[PhpStormStubsElementAvailable(from: '8.0')] array $unserializeOptions = []): mixed
    {
    }

    /**
     * (PHP &gt;= 5.3.0, PECL phar &gt;= 1.0.0)<br/>
     * Return whether phar was modified
     * @link https://php.net/manual/en/phar.getmodified.php
     * @return bool <b>TRUE</b> if the phar has been modified since opened, <b>FALSE</b> if not.
     */
    public function getModified(): bool
    {
    }

    /**
     * (PHP &gt;= 5.3.0, PECL phar &gt;= 1.0.0)<br/>
     * Return MD5/SHA1/SHA256/SHA512/OpenSSL signature of a Phar archive
     * @link https://php.net/manual/en/phar.getsignature.php
     * @return array Array with the opened archive's signature in hash key and MD5,
     * SHA-1,
     * SHA-256, SHA-512, or OpenSSL
     * in hash_type. This signature is a hash calculated on the
     * entire phar's contents, and may be used to verify the integrity of the archive.
     * A valid signature is absolutely required of all executable phar archives if the
     * phar.require_hash INI variable
     * is set to true.
     */
    #[ArrayShape(['hash' => 'string', 'hash_type' => 'string'])]
    public function getSignature(): array|false
    {
    }

    /**
     * (PHP &gt;= 5.3.0, PECL phar &gt;= 1.0.0)<br/>
     * Return the PHP loader or bootstrap stub of a Phar archive
     * @link https://php.net/manual/en/phar.getstub.php
     * @return string a string containing the contents of the bootstrap loader (stub) of
     * the current Phar archive.
     */
    public function getStub(): string
    {
    }

    /**
     * (PHP &gt;= 5.3.0, PECL phar &gt;= 1.0.0)<br/>
     * Return version info of Phar archive
     * @link https://php.net/manual/en/phar.getversion.php
     * @return string The opened archive's API version. This is not to be confused with
     * the API version that the loaded phar extension will use to create
     * new phars. Each Phar archive has the API version hard-coded into
     * its manifest. See Phar file format
     * documentation for more information.
     */
    public function getVersion(): string
    {
    }

    /**
     * (PHP &gt;= 5.3.0, PECL phar &gt;= 1.2.0)<br/>
     * Returns whether phar has global meta-data
     * @link https://php.net/manual/en/phar.hasmetadata.php
     * @return bool <b>TRUE</b> if meta-data has been set, and <b>FALSE</b> if not.
     */
    public function hasMetadata(): bool
    {
    }

    /**
     * (PHP &gt;= 5.3.0, PECL phar &gt;= 1.0.0)<br/>
     * Used to determine whether Phar write operations are being buffered, or are flushing directly to disk
     * @link https://php.net/manual/en/phar.isbuffering.php
     * @return bool <b>TRUE</b> if the write operations are being buffer, <b>FALSE</b> otherwise.
     */
    public function isBuffering(): bool
    {
    }

    /**
     * (PHP &gt;= 5.3.0, PECL phar &gt;= 2.0.0)<br/>
     * Returns Phar::GZ or PHAR::BZ2 if the entire phar archive is compressed (.tar.gz/tar.bz and so on)
     * @link https://php.net/manual/en/phar.iscompressed.php
     * @return mixed Phar::GZ, Phar::BZ2 or <b>FALSE</b>
     */
    public function isCompressed(): int|false
    {
    }

    /**
     * (PHP &gt;= 5.3.0, PECL phar &gt;= 2.0.0)<br/>
     * Returns true if the phar archive is based on the tar/phar/zip file format depending on the parameter
     * @link https://php.net/manual/en/phar.isfileformat.php
     * @param int $format <p>
     * Either Phar::PHAR, Phar::TAR, or
     * Phar::ZIP to test for the format of the archive.
     * </p>
     * @return bool <b>TRUE</b> if the phar archive matches the file format requested by the parameter
     */
    public function isFileFormat(int $format): bool
    {
    }

    /**
     * (Unknown)<br/>
     * Returns true if the phar archive can be modified
     * @link https://php.net/manual/en/phar.iswritable.php
     * @return bool <b>TRUE</b> if the phar archive can be modified
     */
    public function isWritable(): bool
    {
    }

    /**
     * (PHP &gt;= 5.3.0, PECL phar &gt;= 1.0.0)<br/>
     * determines whether a file exists in the phar
     * @link https://php.net/manual/en/phar.offsetexists.php
     * @param string $localName <p>
     * The filename (relative path) to look for in a Phar.
     * </p>
     * @return bool <b>TRUE</b> if the file exists within the phar, or <b>FALSE</b> if not.
     */
    public function offsetExists($localName): bool
    {
    }

    /**
     * (PHP &gt;= 5.3.0, PECL phar &gt;= 1.0.0)<br/>
     * Gets a <b>PharFileInfo</b> object for a specific file
     * @link https://php.net/manual/en/phar.offsetget.php
     * @param string $localName <p>
     * The filename (relative path) to look for in a Phar.
     * </p>
     * @return PharFileInfo A <b>PharFileInfo</b> object is returned that can be used to
     * iterate over a file's contents or to retrieve information about the current file.
     */
    public function offsetGet($localName): SplFileInfo
    {
    }

    /**
     * (PHP &gt;= 5.3.0, PECL phar &gt;= 1.0.0)<br/>
     * set the contents of an internal file to those of an external file
     * @link https://php.net/manual/en/phar.offsetset.php
     * @param string $localName <p>
     * The filename (relative path) to modify in a Phar.
     * </p>
     * @param string $value <p>
     * Content of the file.
     * </p>
     * @return void No return values.
     */
    public function offsetSet($localName, $value): void
    {
    }

    /**
     * (PHP &gt;= 5.3.0, PECL phar &gt;= 1.0.0)<br/>
     * remove a file from a phar
     * @link https://php.net/manual/en/phar.offsetunset.php
     * @param string $localName <p>
     * The filename (relative path) to modify in a Phar.
     * </p>
     * @return bool <b>TRUE</b> on success or <b>FALSE</b> on failure.
     */
    public function offsetUnset($localName): void
    {
    }

    /**
     * (PHP &gt;= 5.3.0, PECL phar &gt;= 1.2.1)<br/>
     * Set the alias for the Phar archive
     * @link https://php.net/manual/en/phar.setalias.php
     * @param string $alias <p>
     * A shorthand string that this archive can be referred to in phar
     * stream wrapper access.
     * </p>
     * @return bool
     */
    #[LanguageLevelTypeAware(['8.4' => 'true'], default: 'bool')]
    public function setAlias(string $alias)
    {
    }

    /**
     * (Unknown)<br/>
     * Used to set the PHP loader or bootstrap stub of a Phar archive to the default loader
     * @link https://php.net/manual/en/phar.setdefaultstub.php
     * @param string $index [optional] <p>
     * Relative path within the phar archive to run if accessed on the command-line
     * </p>
     * @param string $webIndex [optional] <p>
     * Relative path within the phar archive to run if accessed through a web browser
     * </p>
     * @return bool <b>TRUE</b> on success or <b>FALSE</b> on failure.
     */
    #[LanguageLevelTypeAware(['8.4' => 'true'], default: 'bool')]
    public function setDefaultStub(string|null $index = null, string|null $webIndex = null)
    {
    }

    /**
     * (PHP &gt;= 5.3.0, PECL phar &gt;= 1.0.0)<br/>
     * Sets phar archive meta-data
     * @link https://php.net/manual/en/phar.setmetadata.php
     * @param mixed $metadata <p>
     * Any PHP variable containing information to store that describes the phar archive
     * </p>
     * @return void No value is returned.
     */
    public function setMetadata(mixed $metadata): void
    {
    }

    /**
     * (PHP &gt;= 5.3.0, PECL phar &gt;= 1.1.0)<br/>
     * set the signature algorithm for a phar and apply it.
     * @link https://php.net/manual/en/phar.setsignaturealgorithm.php
     * @param int $algo <p>
     * One of Phar::MD5,
     * Phar::SHA1, Phar::SHA256,
     * Phar::SHA512, or Phar::OPENSSL
     * </p>
     * @param string $privateKey [optional] <p>
     * The contents of an OpenSSL private key, as extracted from a certificate or
     * OpenSSL key file:
     * <code>
     * $private = openssl_get_privatekey(file_get_contents('private.pem'));
     * $pkey = '';
     * openssl_pkey_export($private, $pkey);
     * $p->setSignatureAlgorithm(Phar::OPENSSL, $pkey);
     * </code>
     * See phar introduction for instructions on
     * naming and placement of the public key file.
     * </p>
     * @return void No value is returned.
     */
    public function setSignatureAlgorithm(int $algo, string|null $privateKey = null): void
    {
    }

    /**
     * (PHP &gt;= 5.3.0, PECL phar &gt;= 1.0.0)<br/>
     * Used to set the PHP loader or bootstrap stub of a Phar archive
     * @link https://php.net/manual/en/phar.setstub.php
     * @param string $stub <p>
     * A string or an open stream handle to use as the executable stub for this
     * phar archive.
     * </p>
     * @param int $length [optional] <p>
     * </p>
     * @return bool <b>TRUE</b> on success or <b>FALSE</b> on failure.
     */
    #[LanguageLevelTypeAware(['8.4' => 'true'], default: 'bool')]
    public function setStub($stub, int $length)
    {
    }

    /**
     * (PHP &gt;= 5.3.0, PECL phar &gt;= 1.0.0)<br/>
     * Start buffering Phar write operations, do not modify the Phar object on disk
     * @link https://php.net/manual/en/phar.startbuffering.php
     * @return void No value is returned.
     */
    public function startBuffering(): void
    {
    }

    /**
     * (PHP &gt;= 5.3.0, PECL phar &gt;= 1.0.0)<br/>
     * Stop buffering write requests to the Phar archive, and save changes to disk
     * @link https://php.net/manual/en/phar.stopbuffering.php
     * @return void No value is returned.
     */
    public function stopBuffering(): void
    {
    }

    /**
     * (PHP &gt;= 5.3.0, PECL phar &gt;= 1.0.0)<br/>
     * Returns the api version
     * @link https://php.net/manual/en/phar.apiversion.php
     * @return string The API version string as in &#x00022;1.0.0&#x00022;.
     */
    final public static function apiVersion(): string
    {
    }

    /**
     * (PHP &gt;= 5.3.0, PECL phar &gt;= 1.0.0)<br/>
     * Returns whether phar extension supports compression using either zlib or bzip2
     * @link https://php.net/manual/en/phar.cancompress.php
     * @param int $compression [optional] <p>
     * Either Phar::GZ or Phar::BZ2 can be
     * used to test whether compression is possible with a specific compression
     * algorithm (zlib or bzip2).
     * </p>
     * @return bool <b>TRUE</b> if compression/decompression is available, <b>FALSE</b> if not.
     */
    final public static function canCompress(int $compression = 0): bool
    {
    }

    /**
     * (PHP &gt;= 5.3.0, PECL phar &gt;= 1.0.0)<br/>
     * Returns whether phar extension supports writing and creating phars
     * @link https://php.net/manual/en/phar.canwrite.php
     * @return bool <b>TRUE</b> if write access is enabled, <b>FALSE</b> if it is disabled.
     */
    final public static function canWrite(): bool
    {
    }

    /**
     * (Unknown)<br/>
     * Create a phar-file format specific stub
     * @link https://php.net/manual/en/phar.createdefaultstub.php
     * @param string|null $index [optional]
     * @param string|null $webIndex [optional]
     * @return string a string containing the contents of a customized bootstrap loader (stub)
     * that allows the created Phar archive to work with or without the Phar extension
     * enabled.
     */
    final public static function createDefaultStub(null|string $index = null, null|string $webIndex = null): string
    {
    }

    /**
     * (PHP &gt;= 5.3.0, PECL phar &gt;= 1.2.0)<br/>
     * Return array of supported compression algorithms
     * @link https://php.net/manual/en/phar.getsupportedcompression.php
     * @return string[] an array containing any of "GZ" or
     * "BZ2", depending on the availability of
     * the zlib extension or the
     * bz2 extension.
     */
    final public static function getSupportedCompression(): array
    {
    }

    /**
     * (PHP &gt;= 5.3.0, PECL phar &gt;= 1.1.0)<br/>
     * Return array of supported signature types
     * @link https://php.net/manual/en/phar.getsupportedsignatures.php
     * @return string[] an array containing any of "MD5", "SHA-1",
     * "SHA-256", "SHA-512", or "OpenSSL".
     */
    final public static function getSupportedSignatures(): array
    {
    }

    /**
     * (PHP &gt;= 5.3.0, PECL phar &gt;= 2.0.0)<br/>
     * instructs phar to intercept fopen, file_get_contents, opendir, and all of the stat-related functions
     * @link https://php.net/manual/en/phar.interceptfilefuncs.php
     * @return void
     */
    final public static function interceptFileFuncs(): void
    {
    }

    /**
     * (PHP &gt;= 5.3.0, PECL phar &gt;= 1.2.0)<br/>
     * Returns whether the given filename is a valid phar filename
     * @link https://php.net/manual/en/phar.isvalidpharfilename.php
     * @param string $filename <p>
     * The name or full path to a phar archive not yet created
     * </p>
     * @param bool $executable [optional] <p>
     * This parameter determines whether the filename should be treated as
     * a phar executable archive, or a data non-executable archive
     * </p>
     * @return bool <b>TRUE</b> if the filename is valid, <b>FALSE</b> if not.
     */
    final public static function isValidPharFilename(string $filename, bool $executable = true): bool
    {
    }

    /**
     * (PHP &gt;= 5.3.0, PECL phar &gt;= 1.0.0)<br/>
     * Loads any phar archive with an alias
     * @link https://php.net/manual/en/phar.loadphar.php
     * @param string $filename <p>
     * the full or relative path to the phar archive to open
     * </p>
     * @param string|null $alias [optional] <p>
     * The alias that may be used to refer to the phar archive. Note
     * that many phar archives specify an explicit alias inside the
     * phar archive, and a <b>PharException</b> will be thrown if
     * a new alias is specified in this case.
     * </p>
     * @return bool <b>TRUE</b> on success or <b>FALSE</b> on failure.
     */
    final public static function loadPhar(string $filename, null|string $alias = null): bool
    {
    }

    /**
     * (PHP &gt;= 5.3.0, PECL phar &gt;= 1.0.0)<br/>
     * Reads the currently executed file (a phar) and registers its manifest
     * @link https://php.net/manual/en/phar.mapphar.php
     * @param string|null $alias [optional] <p>
     * The alias that can be used in phar:// URLs to
     * refer to this archive, rather than its full path.
     * </p>
     * @param int $offset [optional] <p>
     * Unused variable, here for compatibility with PEAR's PHP_Archive.
     * </p>
     * @return bool <b>TRUE</b> on success or <b>FALSE</b> on failure.
     */
    final public static function mapPhar(null|string $alias = null, int $offset = 0): bool
    {
    }

    /**
     * (PHP &gt;= 5.3.0, PECL phar &gt;= 2.0.0)<br/>
     * Returns the full path on disk or full phar URL to the currently executing Phar archive
     * @link https://php.net/manual/en/phar.running.php
     * @param bool $returnPhar <p>
     * If <b>FALSE</b>, the full path on disk to the phar
     * archive is returned. If <b>TRUE</b>, a full phar URL is returned.
     * </p>
     * @return string the filename if valid, empty string otherwise.
     */
    final public static function running(bool $returnPhar = true): string
    {
    }

    /**
     * (PHP &gt;= 5.3.0, PECL phar &gt;= 2.0.0)<br/>
     * Mount an external path or file to a virtual location within the phar archive
     * @link https://php.net/manual/en/phar.mount.php
     * @param string $pharPath <p>
     * The internal path within the phar archive to use as the mounted path location.
     * This must be a relative path within the phar archive, and must not already exist.
     * </p>
     * @param string $externalPath <p>
     * A path or URL to an external file or directory to mount within the phar archive
     * </p>
     * @return void No return. <b>PharException</b> is thrown on failure.
     */
    final public static function mount(string $pharPath, string $externalPath): void
    {
    }

    /**
     * (Unknown)<br/>
     * Defines a list of up to 4 $_SERVER variables that should be modified for execution
     * @link https://php.net/manual/en/phar.mungserver.php
     * @param array $variables <p>
     * an array containing as string indices any of
     * REQUEST_URI, PHP_SELF,
     * SCRIPT_NAME and SCRIPT_FILENAME.
     * Other values trigger an exception, and <b>Phar::mungServer</b>
     * is case-sensitive.
     * </p>
     * @return void No return.
     */
    final public static function mungServer(array $variables): void
    {
    }

    /**
     * (PHP &gt;= 5.3.0, PECL phar &gt;= 2.0.0)<br/>
     * Completely remove a phar archive from disk and from memory
     * @link https://php.net/manual/en/phar.unlinkarchive.php
     * @param string $filename <p>
     * The path on disk to the phar archive.
     * </p>
     * @throws PharException
     * @return bool <b>TRUE</b> on success or <b>FALSE</b> on failure.
     */
    #[LanguageLevelTypeAware(['8.4' => 'true'], default: 'bool')]
    final public static function unlinkArchive(string $filename)
    {
    }

    /**
     * (PHP &gt;= 5.3.0, PECL phar &gt;= 2.0.0)<br/>
     * mapPhar for web-based phars. front controller for web applications
     * @link https://php.net/manual/en/phar.webphar.php
     * @param null|string $alias [optional] <p>
     * The alias that can be used in phar:// URLs to
     * refer to this archive, rather than its full path.
     * </p>
     * @param string|null $index [optional] <p>
     * The location within the phar of the directory index.
     * </p>
     * @param null|string $fileNotFoundScript [optional] <p>
     * The location of the script to run when a file is not found. This
     * script should output the proper HTTP 404 headers.
     * </p>
     * @param null|array $mimeTypes [optional] <p>
     * An array mapping additional file extensions to MIME type.
     * If the default mapping is sufficient, pass an empty array.
     * By default, these extensions are mapped to these MIME types:
     * <code>
     * $mimes = array(
     * 'phps' => Phar::PHPS, // pass to highlight_file()
     * 'c' => 'text/plain',
     * 'cc' => 'text/plain',
     * 'cpp' => 'text/plain',
     * 'c++' => 'text/plain',
     * 'dtd' => 'text/plain',
     * 'h' => 'text/plain',
     * 'log' => 'text/plain',
     * 'rng' => 'text/plain',
     * 'txt' => 'text/plain',
     * 'xsd' => 'text/plain',
     * 'php' => Phar::PHP, // parse as PHP
     * 'inc' => Phar::PHP, // parse as PHP
     * 'avi' => 'video/avi',
     * 'bmp' => 'image/bmp',
     * 'css' => 'text/css',
     * 'gif' => 'image/gif',
     * 'htm' => 'text/html',
     * 'html' => 'text/html',
     * 'htmls' => 'text/html',
     * 'ico' => 'image/x-ico',
     * 'jpe' => 'image/jpeg',
     * 'jpg' => 'image/jpeg',
     * 'jpeg' => 'image/jpeg',
     * 'js' => 'application/x-javascript',
     * 'midi' => 'audio/midi',
     * 'mid' => 'audio/midi',
     * 'mod' => 'audio/mod',
     * 'mov' => 'movie/quicktime',
     * 'mp3' => 'audio/mp3',
     * 'mpg' => 'video/mpeg',
     * 'mpeg' => 'video/mpeg',
     * 'pdf' => 'application/pdf',
     * 'png' => 'image/png',
     * 'swf' => 'application/shockwave-flash',
     * 'tif' => 'image/tiff',
     * 'tiff' => 'image/tiff',
     * 'wav' => 'audio/wav',
     * 'xbm' => 'image/xbm',
     * 'xml' => 'text/xml',
     * );
     * </code>
     * </p>
     * @param null|callable $rewrite [optional] <p>
     * The rewrites function is passed a string as its only parameter and must return a string or <b>FALSE</b>.
     * </p>
     * <p>
     * If you are using fast-cgi or cgi then the parameter passed to the function is the value of the
     * $_SERVER['PATH_INFO'] variable. Otherwise, the parameter passed to the function is the value
     * of the $_SERVER['REQUEST_URI'] variable.
     * </p>
     * <p>
     * If a string is returned it is used as the internal file path. If <b>FALSE</b> is returned then webPhar() will
     * send a HTTP 403 Denied Code.
     * </p>
     * @return void No value is returned.
     */
    final public static function webPhar(
        null|string $alias = null,
        null|string $index = null,
        #[LanguageLevelTypeAware(['8.0' => 'string|null'], default: 'string')]  $fileNotFoundScript = null,
        array $mimeTypes = [],
        null|callable $rewrite = null,
    ): void {
    }

    /**
     * Returns whether current entry is a directory and not '.' or '..'
     * @link https://php.net/manual/en/recursivedirectoryiterator.haschildren.php
     * @param bool $allow_links [optional] <p>
     * </p>
     * @return bool whether the current entry is a directory, but not '.' or '..'
     */
    public function hasChildren($allow_links = false)
    {
    }

    /**
     * Returns an iterator for the current entry if it is a directory
     * @link https://php.net/manual/en/recursivedirectoryiterator.getchildren.php
     * @return mixed The filename, file information, or $this depending on the set flags.
     * See the FilesystemIterator
     * constants.
     */
    public function getChildren()
    {
    }

    /**
     * Rewinds back to the beginning
     * @link https://php.net/manual/en/filesystemiterator.rewind.php
     * @return void No value is returned.
     */
    public function rewind()
    {
    }

    /**
     * Move to the next file
     * @link https://php.net/manual/en/filesystemiterator.next.php
     * @return void No value is returned.
     */
    public function next()
    {
    }

    /**
     * Retrieve the key for the current file
     * @link https://php.net/manual/en/filesystemiterator.key.php
     * @return string the pathname or filename depending on the set flags.
     * See the FilesystemIterator constants.
     */
    public function key()
    {
    }

    /**
     * The current file
     * @link https://php.net/manual/en/filesystemiterator.current.php
     * @return mixed The filename, file information, or $this depending on the set flags.
     * See the FilesystemIterator constants.
     */
    public function current()
    {
    }

    /**
     * Check whether current DirectoryIterator position is a valid file
     * @link https://php.net/manual/en/directoryiterator.valid.php
     * @return bool <b>TRUE</b> if the position is valid, otherwise <b>FALSE</b>
     */
    public function valid()
    {
    }

    /**
     * Seek to a DirectoryIterator item
     * @link https://php.net/manual/en/directoryiterator.seek.php
     * @param int $position <p>
     * The zero-based numeric position to seek to.
     * </p>
     * @return void No value is returned.
     */
    public function seek($position)
    {
    }

    public function _bad_state_ex()
    {
    }
}

/**
 * The PharData class provides a high-level interface to accessing and creating
 * non-executable tar and zip archives. Because these archives do not contain
 * a stub and cannot be executed by the phar extension, it is possible to create
 * and manipulate regular zip and tar files using the PharData class even if
 * phar.readonly php.ini setting is 1.
 * @link https://php.net/manual/en/class.phardata.php
 */
class PharData extends Phar
{
    /**
     * (PHP &gt;= 5.3.0, PECL phar &gt;= 2.0.0)<br/>
     * Construct a non-executable tar or zip archive object
     * @link https://php.net/manual/en/phardata.construct.php
     * @param string $filename <p>
     * Path to an existing tar/zip archive or to-be-created archive
     * </p>
     * @param int $flags [optional] <p>
     * Flags to pass to <b>Phar</b> parent class
     * <b>RecursiveDirectoryIterator</b>.
     * </p>
     * @param string $alias [optional] <p>
     * Alias with which this Phar archive should be referred to in calls to stream
     * functionality.
     * </p>
     * @param int $format [optional] <p>
     * One of the
     * file format constants
     * available within the <b>Phar</b> class.
     * </p>
     */
    public function __construct(
        string $filename,
        int $flags = FilesystemIterator::SKIP_DOTS | FilesystemIterator::UNIX_PATHS,
        string|null $alias = null,
        int $format = 0,
    ) {}

    /**
     * @param string $localName
     * @return bool
     */
    public function offsetExists($localName): bool
    {
    }

    /**
     * @param string $localName
     * @return SplFileInfo
     */
    public function offsetGet($localName): SplFileInfo
    {
    }

    /**
     * (PHP &gt;= 5.3.0, PECL phar &gt;= 2.0.0)<br/>
     * set the contents of a file within the tar/zip to those of an external file or string
     * @link https://php.net/manual/en/phardata.offsetset.php
     * @param string $localName <p>
     * The filename (relative path) to modify in a tar or zip archive.
     * </p>
     * @param string $value <p>
     * Content of the file.
     * </p>
     * @return void No return values.
     */
    public function offsetSet($localName, $value): void
    {
    }

    /**
     * (PHP &gt;= 5.3.0, PECL phar &gt;= 2.0.0)<br/>
     * remove a file from a tar/zip archive
     * @link https://php.net/manual/en/phardata.offsetunset.php
     * @param string $localName <p>
     * The filename (relative path) to modify in the tar/zip archive.
     * </p>
     * @return void
     */
    public function offsetUnset($localName): void
    {
    }

    /**
     * Returns whether current entry is a directory and not '.' or '..'
     * @link https://php.net/manual/en/recursivedirectoryiterator.haschildren.php
     * @param bool $allow_links [optional] <p>
     * </p>
     * @return bool whether the current entry is a directory, but not '.' or '..'
     */
    public function hasChildren($allow_links = false)
    {
    }

    /**
     * Returns an iterator for the current entry if it is a directory
     * @link https://php.net/manual/en/recursivedirectoryiterator.getchildren.php
     * @return mixed The filename, file information, or $this depending on the set flags.
     * See the FilesystemIterator
     * constants.
     */
    public function getChildren()
    {
    }

    /**
     * Rewinds back to the beginning
     * @link https://php.net/manual/en/filesystemiterator.rewind.php
     * @return void No value is returned.
     */
    public function rewind()
    {
    }

    /**
     * Move to the next file
     * @link https://php.net/manual/en/filesystemiterator.next.php
     * @return void No value is returned.
     */
    public function next()
    {
    }

    /**
     * Retrieve the key for the current file
     * @link https://php.net/manual/en/filesystemiterator.key.php
     * @return string the pathname or filename depending on the set flags.
     * See the FilesystemIterator constants.
     */
    public function key()
    {
    }

    /**
     * The current file
     * @link https://php.net/manual/en/filesystemiterator.current.php
     * @return mixed The filename, file information, or $this depending on the set flags.
     * See the FilesystemIterator constants.
     */
    public function current()
    {
    }

    /**
     * Check whether current DirectoryIterator position is a valid file
     * @link https://php.net/manual/en/directoryiterator.valid.php
     * @return bool <b>TRUE</b> if the position is valid, otherwise <b>FALSE</b>
     */
    public function valid()
    {
    }

    /**
     * Seek to a DirectoryIterator item
     * @link https://php.net/manual/en/directoryiterator.seek.php
     * @param int $position <p>
     * The zero-based numeric position to seek to.
     * </p>
     * @return void No value is returned.
     */
    public function seek($position)
    {
    }
}

class PharFileInfo extends SplFileInfo
{
    public function __construct(string $filename) {}

    public function __destruct()
    {
    }

    public function chmod(int $perms): void
    {
    }

    public function compress(int $compression): bool
    {
    }

    public function decompress(): bool
    {
    }

    public function delMetadata(): bool
    {
    }

    public function getCompressedSize(): int
    {
    }

    public function getCRC32(): int
    {
    }

    public function getContent(): string
    {
    }

    public function getMetadata(array $unserializeOptions = []): mixed
    {
    }

    public function getPharFlags(): int
    {
    }

    public function hasMetadata(): bool
    {
    }

    public function isCompressed(int|null $compression = null): bool
    {
    }

    public function isCRCChecked(): bool
    {
    }

    public function setMetadata(mixed $metadata): void
    {
    }
}
