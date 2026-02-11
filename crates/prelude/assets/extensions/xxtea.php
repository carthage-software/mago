<?php

class XXTEA
{
    /**
     * @param string $data
     * @param string $key
     *
     * @return string
     */
    public static function encrypt($data, $key) {}

    /**
     * @param string $data
     * @param string $key
     *
     * @return string|false
     */
    public static function decrypt($data, $key) {}
}

function xxtea_encrypt(string $data, string $key): string {}

function xxtea_decrypt(string $data, string $key): string|false {}
