<?php

declare(strict_types=1);

function one(): void
{
    $n = 0;
    for ($i = 0; $i < $n; $i++) { // @mago-expect analysis:impossible-condition
        $n++;
    }

    if (!$n) { // @mago-expect analysis:redundant-condition
        echo 'nothing';
    }
}

function two(int $n): void
{
    for ($i = 0; $i < $n; $i++) {
        $n++;
    }

    if (!$n) {
        echo 'nothing';
    }
}

/**
 * @param int<0, max> $n
 */
function three(int $n): void
{
    for ($i = 0; $i < $n; $i++) {
        $n++;
    }

    if (!$n) {
        echo 'nothing';
    }
}

/**
 * @param int<1, max> $n
 */
function four(int $n): void
{
    for ($i = 0; $i < $n; $i++) {
        $n++;
    }

    if (!$n) { // @mago-expect analysis:impossible-condition
        echo 'nothing';
    }
}

/**
 * @param int<min, 1> $n
 */
function five(int $n): void
{
    for ($i = 0; $i < $n; $i++) {
        $n++;
    }

    if (!$n) {
        echo 'nothing';
    }
}

/**
 * @param int<min, 0> $n
 */
function six(int $n): void
{
    for ($i = 0; $i < $n; $i++) { // @mago-expect analysis:impossible-condition
        $n++;
    }

    if (!$n) {
        echo 'nothing';
    }
}
