<?php

declare(strict_types=1);

function work(): string
{
    return 'a';
}

function which_catch(): string
{
    try {
        return work();
    } catch (RuntimeException) {
        $marker = 'x';
    } catch (Throwable) {
        $marker = 'y';
    }

    if ($marker === 'x') {
        return 'first';
    }

    return 'second';
}

function set_in_one_catch(): string
{
    try {
        return work();
    } catch (RuntimeException) {
        $detail = 'runtime';
    } catch (Throwable) {
    }

    if (isset($detail)) {
        return $detail;
    }

    return 'none';
}
