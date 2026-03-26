<?php

const EXP_GLOB = 1;

const EXP_EXACT = 2;

const EXP_REGEXP = 3;

const EXP_EOF = -11;

const EXP_TIMEOUT = -2;

const EXP_FULLBUFFER = -5;

/**
 * @return resource|false
 */
function expect_popen(string $command) {}

/**
 * @param resource $expect
 */
function expect_expectl($expect, array $cases, array &$match = []): int {}
