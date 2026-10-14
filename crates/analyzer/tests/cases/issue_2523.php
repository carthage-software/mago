<?php

declare(strict_types=1);

$curl = curl_multi_init();
$stillRunning = 0;

do {
    if ($stillRunning) {
        echo "wait up to 1 s.\n";
        curl_multi_select($curl, 1);
    }

    curl_multi_exec($curl, $stillRunning);
} while ($stillRunning > 0);

function direct_assignment(int $next): void
{
    $running = 0;

    do {
        if ($running) {
            echo 'running';
        }

        $running = $next;
    } while ($running > 0);
}

function update_running(bool &$running): void
{
    $running = (bool) rand(0, 1);
}

function false_on_reentry(): void
{
    $running = true;

    do {
        if ($running) {
            echo 'first iteration';
        }

        update_running($running);
    } while (!$running);
}

function unknown_first_iteration(bool $running): void
{
    do {
        if ($running) {
            echo 'running';
        }

        update_running($running);
    } while ($running);
}

function always_truthy(CurlMultiHandle $curl): void
{
    $running = 1;

    do {
        // @mago-expect analysis:redundant-condition
        if ($running) {
            curl_multi_select($curl, 1);
        }

        curl_multi_exec($curl, $running);
    } while ($running > 0);
}

function always_falsy(): void
{
    $running = false;

    do {
        // @mago-expect analysis:impossible-condition
        if ($running) {
            echo 'unreachable';
        }

        update_running($running);
    } while (!$running);
}
