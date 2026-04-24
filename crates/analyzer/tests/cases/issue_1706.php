<?php

declare(strict_types=1);

function test(string $fn, string $str): void
{
    $is_stderr = false;
    $f = fopen($fn, mode: 'a');
    if (!$f) {
        if ('cli' === PHP_SAPI) {
            $f = fopen('php://stderr', mode: 'a');
        }

        if (!$f) {
            error_log($str);
            return;
        }

        $is_stderr = true;
    }

    if ($is_stderr) {
        fwrite($f, "stderr write\n");
    }

    fclose($f);
}

