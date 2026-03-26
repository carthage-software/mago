<?php

/**
 * @param int $flags
 *
 * @return void
 */
function xhprof_enable($flags = 0, array $options = []) {}

/**
 * @return array
 */
function xhprof_disable() {}

/**
 * @return void
 */
function xhprof_sample_enable() {}

/**
 * @return array
 */
function xhprof_sample_disable() {}

const XHPROF_FLAGS_NO_BUILTINS = 1;

const XHPROF_FLAGS_CPU = 2;

const XHPROF_FLAGS_MEMORY = 4;
