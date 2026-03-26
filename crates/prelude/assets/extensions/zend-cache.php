<?php

/**
 * @param string $key
 * @param int $ttl
 *
 * @return bool
 */
function zend_shm_cache_store($key, $value, $ttl = 0) {}

/**
 * @param string $key
 */
function zend_shm_cache_fetch($key) {}

/**
 * @param string $key
 */
function zend_shm_cache_delete($key) {}

/**
 * @param string $namespace
 *
 * @return bool
 */
function zend_shm_cache_clear($namespace = '') {}

/**
 * @return array|false
 */
function zend_shm_cache_info() {}

/**
 * @param string $key
 * @param int $ttl
 */
function zend_disk_cache_store($key, $value, $ttl = 0) {}

/**
 * @param string $key
 */
function zend_disk_cache_fetch($key) {}

/**
 * @param string $key
 */
function zend_disk_cache_delete($key) {}

/**
 * @param string $namespace
 *
 * @return bool
 */
function zend_disk_cache_clear($namespace = '') {}

/**
 * @return array|false
 */
function zend_disk_cache_info() {}
