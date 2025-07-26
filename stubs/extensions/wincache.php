<?php

/**
 * @param bool $summaryonly
 *
 * @return array|false
 */
function wincache_fcache_fileinfo($summaryonly = false)
{
}

/**
 * @return array|false
 */
function wincache_fcache_meminfo()
{
}

/**
 * @param string $key
 * @param bool $isglobal
 *
 * @return bool
 */
function wincache_lock($key, $isglobal = false)
{
}

/**
 * @param bool $summaryonly
 *
 * @return array|false
 */
function wincache_ocache_fileinfo($summaryonly = false)
{
}

/**
 * @return array|false
 */
function wincache_ocache_meminfo()
{
}

/**
 * @param array $files
 *
 * @return bool
 */
function wincache_refresh_if_changed(array $files = [])
{
}

/**
 * @return array|false
 */
function wincache_rplist_fileinfo()
{
}

/**
 * @return array|false
 */
function wincache_rplist_meminfo()
{
}

/**
 * @param bool $summaryonly
 *
 * @return array|false
 */
function wincache_scache_info($summaryonly = false)
{
}

/**
 * @return array|false
 */
function wincache_scache_meminfo()
{
}

/**
 * @param string $key
 * @param int $ttl
 *
 * @return bool
 */
function wincache_ucache_add($key, $value, $ttl = 0)
{
}

/**
 * @param string $key
 * @param int $old_value
 * @param int $new_value
 *
 * @return bool
 */
function wincache_ucache_cas($key, $old_value, $new_value)
{
}

/**
 * @return bool
 */
function wincache_ucache_clear()
{
}

/**
 * @param string $key
 * @param int $dec_by
 * @param bool|null &$success
 *
 * @return int|false
 */
function wincache_ucache_dec($key, $dec_by = 1, &$success = null)
{
}

/**
 * @param string|string[] $key
 *
 * @return bool
 */
function wincache_ucache_delete($key)
{
}

/**
 * @param string $key
 *
 * @return bool
 */
function wincache_ucache_exists($key)
{
}

/**
 * @param string|string[] $key
 * @param bool|null &$success
 *
 * @return mixed
 */
function wincache_ucache_get($key, &$success = null)
{
}

/**
 * @param string $key
 * @param int $inc_by
 * @param bool|null &$success
 *
 * @return int|false
 */
function wincache_ucache_inc($key, $inc_by = 1, &$success = null)
{
}

/**
 * @param bool $summaryonly
 * @param null|string $key
 *
 * @return array|false
 */
function wincache_ucache_info(bool $summaryonly = false, $key = null)
{
}

/**
 * @return array|false
 */
function wincache_ucache_meminfo()
{
}

/**
 * @param string|string[] $key
 * @param int $ttl
 *
 * @return bool
 */
function wincache_ucache_set($key, $value, $ttl = 0)
{
}

/**
 * @param string $key
 *
 * @return bool
 */
function wincache_unlock($key)
{
}
