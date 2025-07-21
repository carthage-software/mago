<?php

final class HashContext
{
    private function __construct() {}

    public function __serialize(): array
    {
    }

    public function __unserialize(array $data): void
    {
    }

    public function __debugInfo(): array
    {
    }
}

/**
 * @pure
 */
function hash(string $algo, string $data, bool $binary = false, array $options = []): string
{
}

/**
 * @pure
 */
function hash_equals(string $known_string, string $user_string): bool
{
}

/**
 * @pure
 */
function hash_file(string $algo, string $filename, bool $binary = false, array $options = []): string|false
{
}

/**
 * @pure
 */
function hash_hmac(string $algo, string $data, string $key, bool $binary = false): string
{
}

/**
 * @pure
 */
function hash_hmac_file(string $algo, string $filename, string $key, bool $binary = false): string|false
{
}

/**
 * @pure
 */
function hash_init(string $algo, int $flags = 0, string $key = '', array $options = []): HashContext
{
}

function hash_update(HashContext $context, string $data): bool
{
}

/**
 * @param resource $stream
 */
function hash_update_stream(HashContext $context, $stream, int $length = -1): int
{
}

/**
 * @param ?resource $stream_context
 */
function hash_update_file(HashContext $context, string $filename, $stream_context = null): bool
{
}

function hash_final(HashContext $context, bool $binary = false): string
{
}

/**
 * @pure
 */
function hash_copy(HashContext $context): HashContext
{
}

/**
 * @return non-empty-list<non-empty-string>
 * @pure
 */
function hash_algos(): array
{
}

/**
 * @param non-empty-string $key
 * @param int<0, max> $length
 *
 * @pure
 */
function hash_hkdf(string $algo, string $key, int $length = 0, string $info = '', string $salt = ''): string
{
}

/**
 * @return non-empty-list<non-empty-string>
 *
 * @pure
 */
function hash_hmac_algos(): array
{
}

/**
 * @param int<1, max> $iterations
 * @param int<0, max> $length
 *
 * @pure
 */
function hash_pbkdf2(
    string $algo,
    string $password,
    string $salt,
    int $iterations,
    int $length = 0,
    bool $binary = false,
    array $options = [],
): string {
}

/**
 * @param int<0, max> $length
 *
 * @deprecated
 * @pure
 */
function mhash_keygen_s2k(int $algo, string $password, string $salt, int $length): string|false
{
}

/**
 * @deprecated
 * @pure
 */
function mhash_get_block_size(int $algo): int|false
{
}

/**
 * @return string|false
 *
 * @deprecated
 * @pure
 */
function mhash_get_hash_name(int $algo): string|false
{
}

/**
 * @return int<0, max>
 *
 * @deprecated
 * @pure
 */
function mhash_count(): int
{
}

/**
 * @deprecated
 * @pure
 */
function mhash(int $algo, string $data, null|string $key = null): string|false
{
}

const HASH_HMAC = 1;

const MHASH_CRC32 = 0;

const MHASH_CRC32C = 34;

const MHASH_MD5 = 1;

const MHASH_SHA1 = 2;

const MHASH_HAVAL256 = 3;

const MHASH_RIPEMD160 = 5;

const MHASH_TIGER = 7;

const MHASH_GOST = 8;

const MHASH_CRC32B = 9;

const MHASH_HAVAL224 = 10;

const MHASH_HAVAL192 = 11;

const MHASH_HAVAL160 = 12;

const MHASH_HAVAL128 = 13;

const MHASH_TIGER128 = 14;

const MHASH_TIGER160 = 15;

const MHASH_MD4 = 16;

const MHASH_SHA256 = 17;

const MHASH_ADLER32 = 18;

const MHASH_SHA224 = 19;

const MHASH_SHA512 = 20;

const MHASH_SHA384 = 21;

const MHASH_WHIRLPOOL = 22;

const MHASH_RIPEMD128 = 23;

const MHASH_RIPEMD256 = 24;

const MHASH_RIPEMD320 = 25;

const MHASH_SNEFRU256 = 27;

const MHASH_MD2 = 28;

const MHASH_FNV132 = 29;

const MHASH_FNV1A32 = 30;

const MHASH_FNV164 = 31;

const MHASH_FNV1A64 = 32;

const MHASH_JOAAT = 33;

const MHASH_MURMUR3A = 35;

const MHASH_MURMUR3C = 36;

const MHASH_MURMUR3F = 37;

const MHASH_XXH32 = 38;

const MHASH_XXH64 = 39;

const MHASH_XXH3 = 40;

const MHASH_XXH128 = 41;
