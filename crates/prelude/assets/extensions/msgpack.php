<?php

const MESSAGEPACK_OPT_PHPONLY = -1001;

/**
 * @return string
 */
function msgpack_serialize($value) {}

/**
 * @param string $str
 * @param null|array|string|object $object
 */
function msgpack_unserialize($str, $object = null) {}

/**
 * @return string
 */
function msgpack_pack($value) {}

/**
 * @param string $str
 * @param null|array|string|object $object
 */
function msgpack_unpack($str, $object = null) {}

class MessagePack
{
    public const OPT_PHPONLY = -1001;

    public function __construct($opt) {}

    public function setOption($option, $value) {}

    public function pack($value) {}

    public function unpack($str, $object) {}

    public function unpacker() {}
}

class MessagePackUnpacker
{
    public function __construct($opt) {}

    public function __destruct() {}

    public function setOption($option, $value) {}

    public function feed($str) {}

    public function execute($str, &$offset) {}

    public function data($object) {}

    public function reset() {}
}
