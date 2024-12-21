<?php
/**
 * @since 8.4
 */
function sodium_crypto_aead_aegis128l_decrypt(string $ciphertext, string $additional_data, string $nonce, #[\SensitiveParameter] string $key): string|false {}
/**
 * @since 8.4
 */
function sodium_crypto_aead_aegis128l_encrypt(#[\SensitiveParameter] string $message, string $additional_data, string $nonce, #[\SensitiveParameter] string $key): string {}
/**
 * @since 8.4
 */
function sodium_crypto_aead_aegis128l_keygen(): string {}
/**
 * @since 8.4
 */
function sodium_crypto_aead_aegis256_decrypt(string $ciphertext, string $additional_data, string $nonce, #[\SensitiveParameter] string $key): string|false {}
/**
 * @since 8.4
 */
function sodium_crypto_aead_aegis256_encrypt(#[\SensitiveParameter] string $message, string $additional_data, string $nonce, #[\SensitiveParameter] string $key): string {}
/**
 * @since 8.4
 */
function sodium_crypto_aead_aegis256_keygen(): string {}
