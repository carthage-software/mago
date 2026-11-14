<?php

declare(strict_types=1);

/**
 * @throws RuntimeException
 */
function loop_exit_connect(): int
{
    return rand(0, 9) ?: throw new RuntimeException('down');
}

function loop_exit_connection(): int
{
    while (!isset($conn)) {
        try {
            $conn = loop_exit_connect();
        } catch (RuntimeException) {
            sleep(1);
        }
    }

    return $conn;
}

function loop_exit_connection_for(): int
{
    for (; !isset($conn);) {
        try {
            $conn = loop_exit_connect();
        } catch (RuntimeException) {
            sleep(1);
        }
    }

    return $conn;
}
