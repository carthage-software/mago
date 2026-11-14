<?php

declare(strict_types=1);

function do_while_first_pass_count_up(): void
{
    $i = 0;
    do {
        if ($i > 0) {
            echo $i;
        }
        $i++;
    } while ($i < 10);
}

function do_while_first_pass_lengths(): void
{
    $previous = null;
    do {
        if ($previous !== null) {
            echo strlen($previous);
        }
        $previous = 'abc';
    } while (rand(0, 1) === 1);
}

function do_while_first_pass_else_branch(): void
{
    $n = 0;
    do {
        if ($n === 0) {
            echo 'first';
        } else {
            echo 'later';
        }

        $n++;
    } while ($n < 5);
}

function do_while_first_pass_decrement(): void
{
    $depth = 0;
    do {
        if ($depth > 0) {
            $depth--;
        } else {
            $depth += 2;
        }
    } while (rand(0, 1) === 1);
}

function do_while_first_pass_match_arms(): void
{
    $n = 0;
    do {
        echo match ($n) {
            0 => 'zero',
            default => 'more',
        };

        $n++;
    } while ($n < 5);
}

function do_while_first_pass_switch_cases(): void
{
    $n = 0;
    do {
        switch ($n) {
            case 0:
                echo 'zero';
                break;
            case 1:
                echo 'one';
                break;
            default:
                echo 'more';
        }

        $n++;
    } while ($n < 5);
}

function do_while_first_pass_coalesce(): void
{
    $s = null;
    do {
        echo $s ?? 'none';
        $s = 'x';
    } while (rand(0, 1) === 1);
}

function do_while_first_pass_narrowed(): void
{
    $s = null;
    do {
        if ($s !== null) {
            echo strlen($s);
        }

        $s = 'x';
    } while (rand(0, 1) === 1);
}

function do_while_first_pass_logical(): void
{
    $out = [];
    do {
        if ($out !== [] && rand(0, 1) === 1) {
            echo count($out);
        }

        $out[] = 1;
    } while (rand(0, 1) === 1);
}
