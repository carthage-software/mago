<?php

declare(strict_types=1);

function test(string $rdir, string $fn): bool
{
    $n = '';
    $tmpfilename = '';
    $fdout = false;
    do {
        $tmpfilename = "{$rdir}/{$fn}.tmp{$n}";
        $fdout = fopen($tmpfilename, mode: 'x'); // x: like w, but do not overwrite
        if (false === $fdout) {
            echo "failed to open {$tmpfilename} for writing\n";
        }
        $n = intval($n) + 1;
    } while (!$fdout && $n < 10);

    if ($fdout) {
        fwrite($fdout, data: 'anything');
        fclose($fdout);
        return true;
    }

    return false;
}
