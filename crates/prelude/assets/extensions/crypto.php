<?php

namespace Crypto;

use Exception;

class Cipher
{
    public const MODE_ECB = 1;
    public const MODE_CBC = 2;
    public const MODE_CFB = 3;
    public const MODE_OFB = 4;
    public const MODE_CTR = 5;
    public const MODE_GCM = 6;
    public const MODE_CCM = 7;
    public const MODE_XTS = 65537;

    /**
     * @param bool $aliases
     * @param string $prefix
     * @return string
     */
    public static function getAlgorithms($aliases = false, $prefix = null) {}

    /**
     * @param string $algorithm
     * @return bool
     */
    public static function hasAlgorithm($algorithm) {}

    /**
     * @param int $mode
     * @return bool
     */
    public static function hasMode($mode) {}

    /**
     * @param string $name
     * @param array $arguments
     */
    public static function __callStatic($name, $arguments) {}

    /**
     * @param string $algorithm
     * @param int $mode
     * @param string $key_size
     */
    public function __construct($algorithm, $mode = null, $key_size = null) {}

    /**
     * @return string
     */
    public function getAlgorithmName() {}

    /**
     * @param string $key
     * @param string $iv
     * @return bool
     */
    public function encryptInit($key, $iv = null) {}

    /**
     * @param string $data
     * @return string
     */
    public function encryptUpdate($data) {}

    /**
     * @return string
     */
    public function encryptFinish() {}

    /**
     * @param string $data
     * @param string $key
     * @param string $iv
     * @return string
     */
    public function encrypt($data, $key, $iv = null) {}

    /**
     * @param string $key
     * @param string $iv
     * @return null
     */
    public function decryptInit($key, $iv = null) {}

    /**
     * @param string $data
     * @return string
     */
    public function decryptUpdate($data) {}

    /**
     * @return string
     */
    public function decryptFinish() {}

    /**
     * @param string $data
     * @param string $key
     * @param string $iv
     * @return string
     */
    public function decrypt($data, $key, $iv = null) {}

    /**
     * @return int
     */
    public function getBlockSize() {}

    /**
     * @return int
     */
    public function getKeyLength() {}

    /**
     * @return int
     */
    public function getIVLength() {}

    /**
     * @return int
     */
    public function getMode() {}

    /**
     * @return string
     */
    public function getTag() {}

    /**
     * Sets authentication tag
     * @param string $tag
     * @return bool
     */
    public function setTag($tag) {}

    /**
     * @param int $tag_length
     * @return bool
     */
    public function setTagLength($tag_length) {}

    /**
     * @param string $aad
     * @return bool
     */
    public function setAAD($aad) {}
}

class CipherException extends Exception
{
    public const ALGORITHM_NOT_FOUND = 1;

    public const STATIC_METHOD_NOT_FOUND = 2;

    public const STATIC_METHOD_TOO_MANY_ARGS = 3;

    public const MODE_NOT_FOUND = 4;

    public const MODE_NOT_AVAILABLE = 5;

    public const AUTHENTICATION_NOT_SUPPORTED = 6;

    public const KEY_LENGTH_INVALID = 7;

    public const IV_LENGTH_INVALID = 8;

    public const AAD_SETTER_FORBIDDEN = 9;

    public const AAD_SETTER_FAILED = 10;

    public const AAD_LENGTH_HIGH = 11;

    public const TAG_GETTER_FORBIDDEN = 12;

    public const TAG_SETTER_FORBIDDEN = 13;

    public const TAG_GETTER_FAILED = 14;

    public const TAG_SETTER_FAILED = 15;

    public const TAG_LENGTH_SETTER_FORBIDDEN = 16;

    public const TAG_LENGTH_LOW = 17;

    public const TAG_LENGTH_HIGH = 18;

    public const TAG_VERIFY_FAILED = 19;

    public const INIT_ALG_FAILED = 20;

    public const INIT_CTX_FAILED = 21;

    public const INIT_ENCRYPT_FORBIDDEN = 22;

    public const INIT_DECRYPT_FORBIDDEN = 23;

    public const UPDATE_FAILED = 24;

    public const UPDATE_ENCRYPT_FORBIDDEN = 25;

    public const UPDATE_DECRYPT_FORBIDDEN = 26;

    public const FINISH_FAILED = 27;

    public const FINISH_ENCRYPT_FORBIDDEN = 28;

    public const FINISH_DECRYPT_FORBIDDEN = 29;

    public const INPUT_DATA_LENGTH_HIGH = 30;
}

class Hash
{
    /**
     * @param bool $aliases
     * @param string $prefix
     * @return string
     */
    public static function getAlgorithms($aliases = false, $prefix = null) {}

    /**
     * @param string $algorithm
     * @return bool
     */
    public static function hasAlgorithm($algorithm) {}

    /**
     * @param string $name
     * @param array $arguments
     */
    public static function __callStatic($name, $arguments) {}

    /**
     * @param string $algorithm
     */
    public function __construct($algorithm) {}

    /**
     * @return string
     */
    public function getAlgorithmName() {}

    /**
     * @param string $data
     * @return null
     */
    public function update($data) {}

    /**
     * @return string
     */
    public function digest() {}

    /**
     * @return string
     */
    public function hexdigest() {}

    /**
     * @return int
     */
    public function getBlockSize() {}

    /**
     * @return int
     */
    public function getSize() {}
}

class HashException extends Exception
{
    public const HASH_ALGORITHM_NOT_FOUND = 1;

    public const STATIC_METHOD_NOT_FOUND = 2;

    public const STATIC_METHOD_TOO_MANY_ARGS = 3;

    public const INIT_FAILED = 4;

    public const UPDATE_FAILED = 5;

    public const DIGEST_FAILED = 6;

    public const INPUT_DATA_LENGTH_HIGH = 7;
}

abstract class MAC extends Hash
{
    /**
     * @param string $algorithm
     * @param string $key
     */
    public function __construct($algorithm, $key) {}
}

class MACException extends HashException
{
    public const MAC_ALGORITHM_NOT_FOUND = 1;

    public const KEY_LENGTH_INVALID = 2;
}

class HMAC extends MAC {}

class CMAC extends MAC {}

abstract class KDF
{
    /**
     * @param int $length
     * @param string $salt
     */
    public function __construct($length, $salt = null) {}

    /**
     * @return int
     */
    public function getLength() {}

    /**
     * @param int $length
     * @return bool
     */
    public function setLength($length) {}

    /**
     * @return string
     */
    public function getSalt() {}

    /**
     * @param string $salt
     * @return bool
     */
    public function setSalt($salt) {}
}

class KDFException
{
    public const KEY_LENGTH_LOW = 1;

    public const KEY_LENGTH_HIGH = 2;

    public const SALT_LENGTH_HIGH = 3;

    public const PASSWORD_LENGTH_INVALID = 4;

    public const DERIVATION_FAILED = 5;
}

class PBKDF2 extends KDF
{
    /**
     * @param string $hashAlgorithm
     * @param int $length
     * @param string $salt
     * @param int $iterations
     */
    public function __construct($hashAlgorithm, $length, $salt = null, $iterations = 1000) {}

    /**
     * @param string $password
     * @return string
     */
    public function derive($password) {}

    /**
     * @return int
     */
    public function getIterations() {}

    /**
     * @param int $iterations
     * @return bool
     */
    public function setIterations($iterations) {}

    /**
     * @return string
     */
    public function getHashAlgorithm() {}

    /**
     * @param string $hashAlgorithm
     * @return bool
     */
    public function setHashAlgorithm($hashAlgorithm) {}
}

class PBKDF2Exception extends KDFException
{
    public const HASH_ALGORITHM_NOT_FOUND = 1;

    public const ITERATIONS_HIGH = 2;
}

class Base64
{
    /**
     * @param string $data
     * @return string
     */
    public function encode($data) {}

    /**
     * @param string $data
     * @return string
     */
    public function decode($data) {}

    public function __construct() {}

    /**
     * @param string $data
     */
    public function encodeUpdate($data) {}

    public function encodeFinish() {}

    /**
     * @param string $data
     */
    public function decodeUpdate($data) {}

    public function decodeFinish() {}
}

class Base64Exception extends Exception
{
    public const ENCODE_UPDATE_FORBIDDEN = 1;

    public const ENCODE_FINISH_FORBIDDEN = 2;

    public const DECODE_UPDATE_FORBIDDEN = 3;

    public const DECODE_FINISH_FORBIDDEN = 4;

    public const DECODE_UPDATE_FAILED = 5;

    public const INPUT_DATA_LENGTH_HIGH = 6;
}

class Rand
{
    /**
     * @param int $num
     * @param bool $must_be_strong
     * @param bool &$returned_strong_result
     * @return string
     */
    public static function generate($num, $must_be_strong = true, &$returned_strong_result = true) {}

    /**
     * @param string $buf
     * @param float $entropy
     * @return null
     */
    public static function seed($buf, $entropy = null) {}

    /**
     * @return null
     */
    public static function cleanup() {}

    /**
     * @param string $filename
     * @param int $max_bytes
     * @return int
     */
    public static function loadFile($filename, $max_bytes = -1) {}

    /**
     * @param string $filename
     * @return int
     */
    public static function writeFile($filename) {}
}

class RandException extends Exception
{
    public const GENERATE_PREDICTABLE = 1;
    public const FILE_WRITE_PREDICTABLE = 2;
    public const REQUESTED_BYTES_NUMBER_TOO_HIGH = 3;
    public const SEED_LENGTH_TOO_HIGH = 4;
}
