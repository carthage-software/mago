<?php

/**
 * @return bool
 */
function apache_child_terminate() {}

/**
 * @return array
 *
 * @pure
 */
function apache_get_modules() {}

/**
 * @return string|false
 *
 * @pure
 */
function apache_get_version() {}

/**
 * @param string $variable
 * @param bool $walk_to_top
 *
 * @return string|false
 *
 * @pure
 */
function apache_getenv($variable, $walk_to_top = false) {}

/**
 * @param string $filename
 *
 * @return object
 */
function apache_lookup_uri($filename) {}

/**
 * @param string $note_name
 * @param string $note_value
 *
 * @return string|false
 */
function apache_note($note_name, $note_value = '') {}

/**
 * @return bool
 */
function apache_reset_timeout() {}

/**
 * @return array|false
 */
function apache_response_headers() {}

/**
 * @param string $variable
 * @param string $value
 * @param bool $walk_to_top
 *
 * @return bool
 */
function apache_setenv($variable, $value, $walk_to_top = false) {}

/**
 * @param string $filename
 * @return bool
 */
function virtual($filename) {}
