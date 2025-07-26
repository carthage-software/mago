<?php

/**
 * @param string $name
 * @return mixed
 */
function xcache_get($name)
{
}

/**
 * @param string $name
 * @param mixed $value
 * @param int $ttl
 * @return bool
 */
function xcache_set($name, $value, $ttl = 0)
{
}

/**
 * @param string $name
 * @return bool
 */
function xcache_isset($name)
{
}

/**
 * @param string $name Key name
 * @return bool
 */
function xcache_unset($name)
{
}

/**
 * @param string $prefix Keys' prefix
 * @return bool
 */
function xcache_unset_by_prefix($prefix)
{
}

/**
 * @param string $name
 * @param mixed $value
 * @param int $ttl
 * @return int
 */
function xcache_inc($name, $value = 1, $ttl = 0)
{
}

/**
 * @param string $name
 * @param mixed $value
 * @param int $ttl
 * @return int
 */
function xcache_dec($name, $value = 1, $ttl = 0)
{
}

/**
 * @param int $type
 * @return int
 */
function xcache_count($type)
{
}

/**
 * @param int $type
 * @param int $id
 * @return array
 */
function xcache_info($type, $id)
{
}

/**
 * @param int $type
 * @param int $id
 * @return array
 */
function xcache_list($type, $id)
{
}

/**
 * @param int $type
 * @param int $id
 * @return void
 */
function xcache_clear_cache($type, $id = -1)
{
}

/**
 * @param int $op_type
 * @return string
 */
function xcache_coredump($op_type)
{
}

/**
 * @param string $data
 * @return array
 */
function xcache_coverager_decode($data)
{
}

/**
 * @param bool $clean
 * @return void
 */
function xcache_coverager_start($clean = true)
{
}

/**
 * @param bool $clean
 * @return void
 */
function xcache_coverager_stop($clean = false)
{
}

/**
 * @param bool $clean
 * @return array
 */
function xcache_coverager_get($clean = false)
{
}

/**
 * @param string $filename
 * @return string
 */
function xcache_asm($filename)
{
}

/**
 * @param string $filename
 * @return string
 */
function xcache_dasm_file($filename)
{
}

/**
 * @param string $code
 * @return string
 */
function xcache_dasm_string($code)
{
}

/**
 * @param string $filename
 * @return string
 */
function xcache_encode($filename)
{
}

/**
 * @param string $filename
 * @return bool
 */
function xcache_decode($filename)
{
}

/**
 * @param int $op_type
 * @return string
 */
function xcache_get_op_type($op_type)
{
}

/**
 * @param int $type
 * @return string
 */
function xcache_get_data_type($type)
{
}

/**
 * @param int $opcode
 * @return string
 */
function xcache_get_opcode($opcode)
{
}

/**
 * @param int $op_type
 * @return string
 */
function xcache_get_op_spec($op_type)
{
}

/**
 * @param int $opcode
 * @return string
 */
function xcache_get_opcode_spec($opcode)
{
}

/**
 * @param string $name
 * @return string
 */
function xcache_is_autoglobal($name)
{
}
