<?php

namespace {
    const SODIUM_CRYPTO_AEAD_AEGIS128L_KEYBYTES = 16;

    const SODIUM_CRYPTO_AEAD_AEGIS128L_NSECBYTES = 0;

    const SODIUM_CRYPTO_AEAD_AEGIS128L_NPUBBYTES = 16;

    const SODIUM_CRYPTO_AEAD_AEGIS128L_ABYTES = 32;

    const SODIUM_CRYPTO_AEAD_AEGIS256_KEYBYTES = 32;

    const SODIUM_CRYPTO_AEAD_AEGIS256_NSECBYTES = 0;

    const SODIUM_CRYPTO_AEAD_AEGIS256_NPUBBYTES = 32;

    const SODIUM_CRYPTO_AEAD_AEGIS256_ABYTES = 32;

    function sodium_crypto_aead_aegis128l_decrypt(
        string $ciphertext,
        string $additional_data,
        string $nonce,
        #[SensitiveParameter] string $key,
    ): string|false {}

    function sodium_crypto_aead_aegis128l_encrypt(
        #[SensitiveParameter] string $message,
        string $additional_data,
        string $nonce,
        #[SensitiveParameter] string $key,
    ): string {}

    function sodium_crypto_aead_aegis128l_keygen(): string {}

    function sodium_crypto_aead_aegis256_decrypt(
        string $ciphertext,
        string $additional_data,
        string $nonce,
        #[SensitiveParameter] string $key,
    ): string|false {}

    function sodium_crypto_aead_aegis256_encrypt(
        #[SensitiveParameter] string $message,
        string $additional_data,
        string $nonce,
        #[SensitiveParameter] string $key,
    ): string {}

    function sodium_crypto_aead_aegis256_keygen(): string {}
}

namespace Sodium {
    const CRYPTO_AEAD_AES256GCM_KEYBYTES = 32;

    const CRYPTO_AEAD_AES256GCM_NSECBYTES = 0;

    const CRYPTO_AEAD_AES256GCM_NPUBBYTES = 12;

    const CRYPTO_AEAD_AES256GCM_ABYTES = 16;

    const CRYPTO_AEAD_CHACHA20POLY1305_KEYBYTES = 32;

    const CRYPTO_AEAD_CHACHA20POLY1305_NSECBYTES = 0;

    const CRYPTO_AEAD_CHACHA20POLY1305_NPUBBYTES = 8;

    const CRYPTO_AEAD_CHACHA20POLY1305_ABYTES = 16;

    const CRYPTO_AUTH_BYTES = 32;

    const CRYPTO_AUTH_KEYBYTES = 32;

    const CRYPTO_BOX_SEALBYTES = 16;

    const CRYPTO_BOX_SECRETKEYBYTES = 32;

    const CRYPTO_BOX_PUBLICKEYBYTES = 32;

    const CRYPTO_BOX_KEYPAIRBYTES = 64;

    const CRYPTO_BOX_MACBYTES = 16;

    const CRYPTO_BOX_NONCEBYTES = 24;

    const CRYPTO_BOX_SEEDBYTES = 32;

    const CRYPTO_KX_BYTES = 32;

    const CRYPTO_KX_PUBLICKEYBYTES = 32;

    const CRYPTO_KX_SECRETKEYBYTES = 32;

    const CRYPTO_GENERICHASH_BYTES = 32;

    const CRYPTO_GENERICHASH_BYTES_MIN = 16;

    const CRYPTO_GENERICHASH_BYTES_MAX = 64;

    const CRYPTO_GENERICHASH_KEYBYTES = 32;

    const CRYPTO_GENERICHASH_KEYBYTES_MIN = 16;

    const CRYPTO_GENERICHASH_KEYBYTES_MAX = 64;

    const CRYPTO_PWHASH_SCRYPTSALSA208SHA256_SALTBYTES = 32;

    const CRYPTO_PWHASH_SCRYPTSALSA208SHA256_STRPREFIX = '$7$';

    const CRYPTO_PWHASH_SCRYPTSALSA208SHA256_OPSLIMIT_INTERACTIVE = 534288;

    const CRYPTO_PWHASH_SCRYPTSALSA208SHA256_MEMLIMIT_INTERACTIVE = 16777216;

    const CRYPTO_PWHASH_SCRYPTSALSA208SHA256_OPSLIMIT_SENSITIVE = 33554432;

    const CRYPTO_PWHASH_SCRYPTSALSA208SHA256_MEMLIMIT_SENSITIVE = 1073741824;

    const CRYPTO_SCALARMULT_BYTES = 32;

    const CRYPTO_SCALARMULT_SCALARBYTES = 32;

    const CRYPTO_SHORTHASH_BYTES = 8;

    const CRYPTO_SHORTHASH_KEYBYTES = 16;

    const CRYPTO_SECRETBOX_KEYBYTES = 32;

    const CRYPTO_SECRETBOX_MACBYTES = 16;

    const CRYPTO_SECRETBOX_NONCEBYTES = 24;

    const CRYPTO_SIGN_BYTES = 64;

    const CRYPTO_SIGN_SEEDBYTES = 32;

    const CRYPTO_SIGN_PUBLICKEYBYTES = 32;

    const CRYPTO_SIGN_SECRETKEYBYTES = 64;

    const CRYPTO_SIGN_KEYPAIRBYTES = 96;

    const CRYPTO_STREAM_KEYBYTES = 32;

    const CRYPTO_STREAM_NONCEBYTES = 24;

    const CRYPTO_PWHASH_OPSLIMIT_INTERACTIVE = 4;

    const CRYPTO_PWHASH_MEMLIMIT_INTERACTIVE = 33554432;

    const CRYPTO_PWHASH_OPSLIMIT_MODERATE = 6;

    const CRYPTO_PWHASH_MEMLIMIT_MODERATE = 134217728;

    const CRYPTO_PWHASH_OPSLIMIT_SENSITIVE = 8;

    const CRYPTO_PWHASH_MEMLIMIT_SENSITIVE = 536870912;

    function crypto_aead_aes256gcm_is_available(): bool {}

    function crypto_aead_aes256gcm_decrypt(string $msg, string $nonce, string $key, string $ad = ''): string {}

    function crypto_aead_aes256gcm_encrypt(string $msg, string $nonce, string $key, string $ad = ''): string {}

    function crypto_aead_chacha20poly1305_decrypt(string $msg, string $nonce, string $key, string $ad = ''): string {}

    function crypto_aead_chacha20poly1305_encrypt(string $msg, string $nonce, string $key, string $ad = ''): string {}

    function crypto_auth(string $msg, string $key): string {}

    function crypto_auth_verify(string $mac, string $msg, string $key): bool {}

    function crypto_box(string $msg, string $nonce, string $keypair): string {}

    function crypto_box_keypair(): string {}

    function crypto_box_seed_keypair(string $seed): string {}

    function crypto_box_keypair_from_secretkey_and_publickey(string $secretkey, string $publickey): string {}

    function crypto_box_open(string $msg, string $nonce, string $keypair): string {}

    function crypto_box_publickey(string $keypair): string {}

    function crypto_box_publickey_from_secretkey(string $secretkey): string {}

    function crypto_box_seal(string $message, string $publickey): string {}

    function crypto_box_seal_open(string $encrypted, string $keypair): string {}

    function crypto_box_secretkey(string $keypair): string {}

    function crypto_kx(
        string $secretkey,
        string $publickey,
        string $client_publickey,
        string $server_publickey,
    ): string {}

    function crypto_generichash(string $input, string $key = '', int $length = 32): string {}

    function crypto_generichash_init(string $key = '', int $length = 32): string {}

    function crypto_generichash_update(string &$hashState, string $append): bool {}

    function crypto_generichash_final(string $state, int $length = 32): string {}

    function crypto_pwhash(int $out_len, string $passwd, string $salt, int $opslimit, int $memlimit): string {}

    function crypto_pwhash_str(string $passwd, int $opslimit, int $memlimit): string {}

    function crypto_pwhash_str_verify(string $hash, string $passwd): bool {}

    function crypto_pwhash_scryptsalsa208sha256(
        int $out_len,
        string $passwd,
        string $salt,
        int $opslimit,
        int $memlimit,
    ): string {}

    function crypto_pwhash_scryptsalsa208sha256_str(string $passwd, int $opslimit, int $memlimit): string {}

    function crypto_pwhash_scryptsalsa208sha256_str_verify(string $hash, string $passwd): bool {}

    function crypto_scalarmult(string $ecdhA, string $ecdhB): string {}

    function crypto_secretbox(string $plaintext, string $nonce, string $key): string {}

    function crypto_secretbox_open(string $ciphertext, string $nonce, string $key): string {}

    function crypto_shorthash(string $message, string $key): string {}

    function crypto_sign(string $message, string $secretkey): string {}

    function crypto_sign_detached(string $message, string $secretkey): string {}

    function crypto_sign_ed25519_pk_to_curve25519(string $sign_pk): string {}

    function crypto_sign_ed25519_sk_to_curve25519(string $sign_sk): string {}

    function crypto_sign_keypair(): string {}

    function crypto_sign_keypair_from_secretkey_and_publickey(string $secretkey, string $publickey): string {}

    function crypto_sign_open(string $signed_message, string $publickey): string {}

    function crypto_sign_publickey(string $keypair): string {}

    function crypto_sign_secretkey(string $keypair): string {}

    function crypto_sign_publickey_from_secretkey(string $secretkey): string {}

    function crypto_sign_seed_keypair(string $seed): string {}

    function crypto_sign_verify_detached(string $signature, string $msg, string $publickey): bool {}

    function crypto_stream(int $length, string $nonce, string $key): string {}

    function crypto_stream_xor(string $plaintext, string $nonce, string $key): string {}

    function randombytes_buf(int $length): string {}

    function randombytes_random16(): int {}

    function randombytes_uniform(int $upperBoundNonInclusive): int {}

    function bin2hex(string $binary): string {}

    function compare(string $left, string $right): int {}

    function hex2bin(string $hex): string {}

    function increment(string &$nonce) {}

    function add(string &$left, string $right) {}

    function library_version_major(): int {}

    function library_version_minor(): int {}

    function memcmp(string $left, string $right): int {}

    function memzero(string &$target) {}

    function version_string(): string {}

    function crypto_scalarmult_base(string $sk): string {}
}
