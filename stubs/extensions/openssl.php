<?php

/**
 * @deprecated
 */
function openssl_pkey_free(OpenSSLAsymmetricKey $key): void
{
}

function openssl_pkey_new(null|array $options): OpenSSLAsymmetricKey|false
{
}

/**
 * @param-out string $output
 */
function openssl_pkey_export(
    OpenSSLAsymmetricKey|OpenSSLCertificate|array|string $key,
    &$output,
    null|string $passphrase,
    null|array $options,
): bool {
}

function openssl_pkey_export_to_file(
    OpenSSLAsymmetricKey|OpenSSLCertificate|array|string $key,
    string $output_filename,
    null|string $passphrase,
    null|array $options,
): bool {
}

function openssl_pkey_get_private(
    OpenSSLAsymmetricKey|OpenSSLCertificate|array|string $private_key,
    null|string $passphrase = null,
): OpenSSLAsymmetricKey|false {
}

function openssl_pkey_get_public(OpenSSLAsymmetricKey|OpenSSLCertificate|array|string $public_key): OpenSSLAsymmetricKey|false {
}

/**
 * @return array{
 *  'bits': int,
 *  'key': string,
 *  'rsa': array,
 *  'dsa': array,
 *  'dh': array,
 *  'ec': array,
 *  'type': int,
 * }|false
 */
function openssl_pkey_get_details(OpenSSLAsymmetricKey $key): array|false
{
}

/**
 * @deprecated
 */
function openssl_free_key(OpenSSLAsymmetricKey $key): void
{
}

function openssl_get_privatekey(
    OpenSSLAsymmetricKey|OpenSSLCertificate|array|string $private_key,
    null|string $passphrase,
): OpenSSLAsymmetricKey|false {
}

function openssl_get_publickey(OpenSSLAsymmetricKey|OpenSSLCertificate|array|string $public_key): OpenSSLAsymmetricKey|false {
}

function openssl_spki_new(OpenSSLAsymmetricKey $private_key, string $challenge, int $digest_algo = 2): string|false
{
}

function openssl_spki_verify(string $spki): bool
{
}

function openssl_spki_export_challenge(string $spki): string|false
{
}

function openssl_spki_export(string $spki): string|false
{
}

function openssl_x509_read(OpenSSLCertificate|false $certificate): OpenSSLCertificate|false
{
}

function openssl_x509_fingerprint(
    OpenSSLCertificate|false $certificate,
    string $digest_algo = 'sha1',
    bool $binary = false,
): string|false {
}

/**
 * @deprecated
 */
function openssl_x509_free(OpenSSLCertificate $certificate): void
{
}

/**
 * @return array{
 *   'name': string,
 *   'subject': string,
 *   'hash': string,
 *   'issuer': string,
 *   'version': int,
 *   'serialNumber': string,
 *   'serialNumberHex': string,
 *   'validFrom': string,
 *   'validTo': string,
 *   'validFrom_time_t': int,
 *   'validTo_time_t': int,
 *   'alias': string,
 *   'signatureTypeSN': string,
 *   'signatureTypeLN': string,
 *   'signatureTypeNID': int,
 *   'purposes': array,
 *   'extensions': array,
 * }|false
 */
function openssl_x509_parse(OpenSSLCertificate|false $certificate, bool $short_names = true): array|false
{
}

function openssl_x509_checkpurpose(
    OpenSSLCertificate|false $certificate,
    int $purpose,
    array $ca_info = [],
    null|string $untrusted_certificates_file,
): int|bool {
}

function openssl_x509_check_private_key(
    OpenSSLCertificate|false $certificate,
    OpenSSLAsymmetricKey|OpenSSLCertificate|array|string $private_key,
): bool {
}

/**
 * @param-out string $output
 */
function openssl_x509_export(OpenSSLCertificate|false $certificate, &$output, bool $no_text = true): bool
{
}

function openssl_x509_export_to_file(
    OpenSSLCertificate|false $certificate,
    string $output_filename,
    bool $no_text = true,
): bool {
}

function openssl_x509_verify(
    OpenSSLCertificate|false $certificate,
    OpenSSLAsymmetricKey|OpenSSLCertificate|array|string $public_key,
): int {
}

/**
 * @param-out string $output
 */
function openssl_pkcs12_export(
    OpenSSLCertificate|false $certificate,
    &$output,
    OpenSSLAsymmetricKey|OpenSSLCertificate|array|string $private_key,
    string $passphrase,
    array $options = [],
): bool {
}

function openssl_pkcs12_export_to_file(
    OpenSSLCertificate|false $certificate,
    string $output_filename,
    OpenSSLAsymmetricKey|OpenSSLCertificate|array|string $private_key,
    string $passphrase,
    array $options = [],
): bool {
}

/**
 * @param-out array $certificates
 */
function openssl_pkcs12_read(string $pkcs12, &$certificates, string $passphrase): bool
{
}

/**
 * @return OpenSSLCertificateSigningRequest|resource|false the CSR.
 */
function openssl_csr_new(
    array $distinguished_names,
    OpenSSLAsymmetricKey $private_key,
    null|array $options,
    null|array $extra_attributes,
): OpenSSLCertificateSigningRequest|bool {
}

/**
 * @param-out string $output
 */
function openssl_csr_export(OpenSSLCertificateSigningRequest|string $csr, &$output, bool $no_text = true): bool
{
}

function openssl_csr_export_to_file(
    OpenSSLCertificateSigningRequest|string $csr,
    string $output_filename,
    bool $no_text = true,
): bool {
}

function openssl_csr_sign(
    OpenSSLCertificateSigningRequest|string $csr,
    OpenSSLCertificate|string|null $ca_certificate,
    OpenSSLAsymmetricKey|OpenSSLCertificate|array|string $private_key,
    int $days,
    null|array $options,
    int $serial = 0,
    null|string $serial_hex = null,
): OpenSSLCertificate|false {
}

function openssl_csr_get_subject(OpenSSLCertificateSigningRequest|string $csr, bool $short_names = true): array|false
{
}

function openssl_csr_get_public_key(
    OpenSSLCertificateSigningRequest|string $csr,
    bool $short_names = true,
): OpenSSLAsymmetricKey|false {
}

function openssl_digest(string $data, string $digest_algo, bool $binary = false): string|false
{
}

/**
 * @param-out string $tag
 *
 * @return string|false the encrypted string on success or false on failure.
 */
function openssl_encrypt(
    string $data,
    string $cipher_algo,
    string $passphrase,
    int $options = 0,
    string $iv = '',
    &$tag = null,
    string $aad = '',
    int $tag_length = 16,
): string|false {
}

function openssl_decrypt(
    string $data,
    string $cipher_algo,
    string $passphrase,
    int $options = 0,
    string $iv = '',
    string|null $tag = null,
    string $aad = '',
): string|false {
}

function openssl_cipher_iv_length(string $cipher_algo): int|false
{
}

function openssl_cipher_key_length(string $cipher_algo): int|false
{
}

/**
 * @param-out string $signature
 */
function openssl_sign(
    string $data,
    &$signature,
    OpenSSLAsymmetricKey|OpenSSLCertificate|array|string $private_key,
    string|int $algorithm = OPENSSL_ALGO_SHA1,
): bool {
}

function openssl_verify(
    string $data,
    string $signature,
    OpenSSLAsymmetricKey|OpenSSLCertificate|array|string $public_key,
    string|int $algorithm = OPENSSL_ALGO_SHA1,
): int|false {
}

/**
 * @param-out string $sealed_data
 * @param-out array $encrypted_keys
 * @param-out string $iv
 */
function openssl_seal(
    string $data,
    &$sealed_data,
    &$encrypted_keys,
    array $public_key,
    string $cipher_algo,
    &$iv = null,
): int|false {
}

/**
 * @param-out string $output
 */
function openssl_open(
    string $data,
    &$output,
    string $encrypted_key,
    OpenSSLAsymmetricKey|OpenSSLCertificate|array|string $private_key,
    string $cipher_algo,
    null|string $iv,
): bool {
}

function openssl_pbkdf2(
    string $password,
    string $salt,
    int $key_length,
    int $iterations,
    string $digest_algo = 'sha1',
): string|false {
}

function openssl_pkcs7_verify(
    string $input_filename,
    int $flags,
    null|string $signers_certificates_filename,
    array $ca_info = [],
    null|string $untrusted_certificates_filename,
    null|string $content,
    null|string $output_filename,
): int|bool {
}

/**
 * @param OpenSSLCertificate|string|resource $certificate
 */
function openssl_pkcs7_decrypt(
    string $input_filename,
    string $output_filename,
    $certificate,
    OpenSSLAsymmetricKey|OpenSSLCertificate|array|string|null $private_key,
): bool {
}

function openssl_pkcs7_sign(
    string $input_filename,
    string $output_filename,
    OpenSSLCertificate|false $certificate,
    OpenSSLAsymmetricKey|OpenSSLCertificate|array|string $private_key,
    null|array $headers,
    int $flags = PKCS7_DETACHED,
    null|string $untrusted_certificates_filename,
): bool {
}

function openssl_pkcs7_encrypt(
    string $input_filename,
    string $output_filename,
    OpenSSLCertificate|array|string $certificate,
    null|array $headers,
    int $flags = 0,
    int $cipher_algo = OPENSSL_CIPHER_AES_128_CBC,
): bool {
}

/**
 * @param-out string $encrypted_data
 */
function openssl_private_encrypt(
    string $data,
    &$encrypted_data,
    OpenSSLAsymmetricKey|OpenSSLCertificate|array|string $private_key,
    int $padding = OPENSSL_PKCS1_PADDING,
): bool {
}

/**
 * @param-out string $decrypted_data
 */
function openssl_private_decrypt(
    string $data,
    &$decrypted_data,
    OpenSSLAsymmetricKey|OpenSSLCertificate|array|string $private_key,
    int $padding = OPENSSL_PKCS1_PADDING,
): bool {
}

/**
 * @param-out string $encrypted_data
 */
function openssl_public_encrypt(
    string $data,
    &$encrypted_data,
    OpenSSLAsymmetricKey|OpenSSLCertificate|array|string $public_key,
    int $padding = OPENSSL_PKCS1_PADDING,
): bool {
}

/**
 * @param-out string $decrypted_data
 */
function openssl_public_decrypt(
    string $data,
    &$decrypted_data,
    OpenSSLAsymmetricKey|OpenSSLCertificate|array|string $public_key,
    int $padding = OPENSSL_PKCS1_PADDING,
): bool {
}

function openssl_get_md_methods(bool $aliases = false): array
{
}

function openssl_get_cipher_methods(bool $aliases = false): array
{
}

function openssl_dh_compute_key(string $public_key, OpenSSLAsymmetricKey $private_key): string|false
{
}

function openssl_pkey_derive(
    OpenSSLAsymmetricKey|OpenSSLCertificate|array|string $public_key,
    OpenSSLAsymmetricKey|OpenSSLCertificate|array|string $private_key,
    int $key_length = 0,
): string|false {
}

/**
 * @param positive-int $length
 *
 * @param-out bool $strong_result
 */
function openssl_random_pseudo_bytes(int $length, &$strong_result): string
{
}

function openssl_error_string(): string|false
{
}

/**
 * @return array{
 *   'default_cert_file': string,
 *   'default_cert_file_env': string,
 *   'default_cert_dir': string,
 *   'default_cert_dir_env': string,
 *   'default_private_dir': string,
 *   'default_default_cert_area': string,
 *   'ini_cafile': string,
 *   'ini_capath': string,
 * }
 */
function openssl_get_cert_locations(): array
{
}

function openssl_get_curve_names(): array|false
{
}

/**
 * @param-out array $certificates
 */
function openssl_pkcs7_read(string $data, &$certificates): bool
{
}

function openssl_cms_verify(
    string $input_filename,
    int $flags = 0,
    null|string $certificates,
    array $ca_info = [],
    null|string $untrusted_certificates_filename,
    null|string $content,
    null|string $pk7,
    null|string $sigfile,
    int $encoding = OPENSSL_ENCODING_SMIME,
): bool {
}

/**
 * @param resource|string|array $certificate
 */
function openssl_cms_encrypt(
    string $input_filename,
    string $output_filename,
    $certificate,
    null|array $headers,
    int $flags = 0,
    int $encoding = OPENSSL_ENCODING_SMIME,
    int $cipher_algo = OPENSSL_CIPHER_AES_128_CBC,
): bool {
}

function openssl_cms_sign(
    string $input_filename,
    string $output_filename,
    OpenSSLCertificate|string $certificate,
    OpenSSLAsymmetricKey|OpenSSLCertificate|array|string $private_key,
    null|array $headers,
    int $flags = 0,
    int $encoding = OPENSSL_ENCODING_SMIME,
    null|string $untrusted_certificates_filename,
): bool {
}

function openssl_cms_decrypt(
    string $input_filename,
    string $output_filename,
    $certificate,
    $private_key = null,
    int $encoding = OPENSSL_ENCODING_SMIME,
): bool {
}

/**
 * @param-out array $certificates
 */
function openssl_cms_read(string $input_filename, &$certificates): bool
{
}

const OPENSSL_VERSION_TEXT = 'OpenSSL 1.0.0e 6 Sep 2011';

const OPENSSL_VERSION_NUMBER = 268435551;

const X509_PURPOSE_SSL_CLIENT = 1;

const X509_PURPOSE_SSL_SERVER = 2;

const X509_PURPOSE_NS_SSL_SERVER = 3;

const X509_PURPOSE_SMIME_SIGN = 4;

const X509_PURPOSE_SMIME_ENCRYPT = 5;

const X509_PURPOSE_CRL_SIGN = 6;

const X509_PURPOSE_ANY = 7;

const X509_PURPOSE_OCSP_HELPER = 8;

const X509_PURPOSE_TIMESTAMP_SIGN = 9;

const OPENSSL_ALGO_SHA1 = 1;

const OPENSSL_ALGO_MD5 = 2;

const OPENSSL_ALGO_MD4 = 3;

const OPENSSL_ALGO_MD2 = 4;

const OPENSSL_ALGO_DSS1 = 5;

const OPENSSL_ALGO_SHA224 = 6;

const OPENSSL_ALGO_SHA256 = 7;

const OPENSSL_ALGO_SHA384 = 8;

const OPENSSL_ALGO_SHA512 = 9;

const OPENSSL_ALGO_RMD160 = 10;

const PKCS7_DETACHED = 64;

const PKCS7_TEXT = 1;

const PKCS7_NOINTERN = 16;

const PKCS7_NOVERIFY = 32;

const PKCS7_NOCHAIN = 8;

const PKCS7_NOCERTS = 2;

const PKCS7_NOATTR = 256;

const PKCS7_BINARY = 128;

const PKCS7_NOOLDMIMETYPE = 1024;

const PKCS7_NOSIGS = 4;

const OPENSSL_PKCS1_PADDING = 1;

const OPENSSL_SSLV23_PADDING = 2;

const OPENSSL_NO_PADDING = 3;

const OPENSSL_PKCS1_OAEP_PADDING = 4;

const OPENSSL_CIPHER_RC2_40 = 0;

const OPENSSL_CIPHER_RC2_128 = 1;

const OPENSSL_CIPHER_RC2_64 = 2;

const OPENSSL_CIPHER_DES = 3;

const OPENSSL_CIPHER_3DES = 4;

const OPENSSL_KEYTYPE_RSA = 0;

const OPENSSL_KEYTYPE_DSA = 1;

const OPENSSL_KEYTYPE_DH = 2;

const OPENSSL_KEYTYPE_EC = 3;

const OPENSSL_KEYTYPE_X25519 = 4;

const OPENSSL_KEYTYPE_ED25519 = 5;

const OPENSSL_KEYTYPE_X448 = 6;

const OPENSSL_KEYTYPE_ED448 = 7;

const OPENSSL_TLSEXT_SERVER_NAME = 1;

const OPENSSL_CIPHER_AES_128_CBC = 5;

const OPENSSL_CIPHER_AES_192_CBC = 6;

const OPENSSL_CIPHER_AES_256_CBC = 7;

const OPENSSL_RAW_DATA = 1;

const OPENSSL_ZERO_PADDING = 2;

const OPENSSL_DONT_ZERO_PAD_KEY = 4;

const OPENSSL_CMS_DETACHED = 64;

const OPENSSL_CMS_TEXT = 1;

const OPENSSL_CMS_NOINTERN = 16;

const OPENSSL_CMS_NOVERIFY = 32;

const OPENSSL_CMS_NOCERTS = 2;

const OPENSSL_CMS_NOATTR = 256;

const OPENSSL_CMS_BINARY = 128;

const OPENSSL_CMS_NOSIGS = 12;

const OPENSSL_ENCODING_DER = 0;

const OPENSSL_ENCODING_SMIME = 1;

const OPENSSL_ENCODING_PEM = 2;

const OPENSSL_CMS_OLDMIMETYPE = 1024;

const OPENSSL_DEFAULT_STREAM_CIPHERS =
    'ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES128-GCM-SHA256:' .
    'ECDHE-RSA-AES256-GCM-SHA384:ECDHE-ECDSA-AES256-GCM-SHA384:DHE-RSA-AES128-GCM-SHA256:' .
    'DHE-DSS-AES128-GCM-SHA256:kEDH+AESGCM:ECDHE-RSA-AES128-SHA256:ECDHE-ECDSA-AES128-SHA256:' .
    'ECDHE-RSA-AES128-SHA:ECDHE-ECDSA-AES128-SHA:ECDHE-RSA-AES256-SHA384:ECDHE-ECDSA-AES256-SHA384:' .
    'ECDHE-RSA-AES256-SHA:ECDHE-ECDSA-AES256-SHA:DHE-RSA-AES128-SHA256:DHE-RSA-AES128-SHA:' .
    'DHE-DSS-AES128-SHA256:DHE-RSA-AES256-SHA256:DHE-DSS-AES256-SHA:DHE-RSA-AES256-SHA:AES128-GCM-SHA256:' .
    'AES256-GCM-SHA384:AES128:AES256:HIGH:!SSLv2:!aNULL:!eNULL:!EXPORT:!DES:!MD5:!RC4:!ADH';

final class OpenSSLCertificate
{
    private function __construct() {}
}

final class OpenSSLCertificateSigningRequest
{
    private function __construct() {}
}

final class OpenSSLAsymmetricKey
{
    private function __construct() {}
}
