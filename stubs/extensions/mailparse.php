<?php

const MAILPARSE_EXTRACT_OUTPUT = 0;

const MAILPARSE_EXTRACT_STREAM = 1;

const MAILPARSE_EXTRACT_RETURN = 2;

/**
 * @param resource $fp
 *
 * @return string
 */
function mailparse_determine_best_xfer_encoding($fp)
{
}

/**
 * @return resource
 */
function mailparse_msg_create()
{
}

/**
 * @param resource $mimemail
 * @param mixed $filename
 * @param null|callable $callbackfunc
 *
 * @return string|bool
 */
function mailparse_msg_extract_part_file($mimemail, $filename, $callbackfunc = null)
{
}

/**
 * @param resource $mimemail
 * @param string $msgbody
 * @param null|callable $callbackfunc
 * @return void
 */
function mailparse_msg_extract_part($mimemail, $msgbody, $callbackfunc = null)
{
}

/**
 * @param resource $mimemail
 * @param string $filename
 * @param callable $callbackfunc
 *
 * @return string
 */
function mailparse_msg_extract_whole_part_file($mimemail, $filename, $callbackfunc = null)
{
}

/**
 * @param resource $mimemail
 *
 * @return bool
 */
function mailparse_msg_free($mimemail)
{
}

/**
 * @param resource $mimemail
 *
 * @return array
 */
function mailparse_msg_get_part_data($mimemail)
{
}

/**
 * @param resource $mimemail
 * @param string $mimesection
 *
 * @return resource|false
 */
function mailparse_msg_get_part($mimemail, $mimesection)
{
}

/**
 * @param resource $mimemail
 *
 * @return array
 */
function mailparse_msg_get_structure($mimemail)
{
}

/**
 * @param string $filename
 *
 * @return resource|false
 */
function mailparse_msg_parse_file($filename)
{
}

/**
 * @param resource $mimemail
 * @param string $data
 *
 * @return bool
 */
function mailparse_msg_parse($mimemail, $data)
{
}

/**
 * @param string $addresses
 *
 * @return array
 */
function mailparse_rfc822_parse_addresses($addresses)
{
}

/**
 * @param resource $sourcefp
 * @param resource $destfp
 * @param string $encoding
 *
 * @return bool
 */
function mailparse_stream_encode($sourcefp, $destfp, $encoding)
{
}

/**
 * @param resource $fp
 * @return array
 */
function mailparse_uudecode_all($fp)
{
}
