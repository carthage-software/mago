<?php

const UUID_VARIANT_NCS = 0;

const UUID_VARIANT_DCE = 1;

const UUID_VARIANT_MICROSOFT = 2;

const UUID_VARIANT_OTHER = 3;

const UUID_TYPE_DEFAULT = 0;

const UUID_TYPE_DCE = 4;

const UUID_TYPE_NAME = 1;

const UUID_TYPE_TIME = 1;

const UUID_TYPE_SECURITY = 2;

const UUID_TYPE_MD5 = 3;

const UUID_TYPE_RANDOM = 4;

const UUID_TYPE_SHA1 = 5;

const UUID_TYPE_NULL = -1;

const UUID_TYPE_INVALID = -42;

/**
 * @param int $uuid_type
 *
 * @return string
 */
function uuid_create($uuid_type = UUID_TYPE_DEFAULT) {}

/**
 * @param string $uuid
 *
 * @return bool
 */
function uuid_is_valid($uuid) {}

/**
 * @param string $uuid1
 * @param string $uuid2
 *
 * @return int
 */
function uuid_compare($uuid1, $uuid2) {}

/**
 * @param string $uuid
 *
 * @return bool
 */
function uuid_is_null($uuid) {}

/**
 * @param string $uuid_ns
 * @param string $name
 *
 * @return string
 */
function uuid_generate_md5($uuid_ns, $name) {}

/**
 * @param string $uuid_ns
 * @param string $name
 *
 * @return string
 */
function uuid_generate_sha1($uuid_ns, $name) {}

/**
 * @param string $uuid
 * @return int
 */
function uuid_type($uuid) {}

/**
 * @param string $uuid
 * @return int
 */
function uuid_variant($uuid) {}

/**
 * @param string $uuid
 * @return false|int
 */
function uuid_time($uuid) {}

/**
 * @param string $uuid
 * @return false|string
 */
function uuid_mac($uuid) {}

/**
 * @param string $uuid
 * @return false|string
 */
function uuid_parse($uuid) {}

/**
 * @param string $uuid
 * @return false|string
 */
function uuid_unparse($uuid) {}
