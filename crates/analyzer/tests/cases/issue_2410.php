<?php

declare(strict_types=1);

namespace Issue2410;

function page_handler(string $page = ''): string|bool
{
    $found = mt_rand(0, max: 2) ? '' : '+';
    if ($found > '') {
        $c = mt_rand(0, max: 2) ? '' : '+';
    }

    $dirs = ['.', 'local', 'whatever'];
    foreach ($dirs as $dir) {
        $path = "{$dir}/{$page}.php";
        if (file_exists($path)) {
            ob_start();
            try {
                require_once $path;
            } finally {
                $c = ob_get_clean();
            }

            return $c;
        }
    }

    return false;
}

/** @return 'final' */
function overwritePossiblyUndefined(bool $assign): string
{
    if ($assign) {
        $value = 1;
    }

    try {
        echo 'try';
    } finally {
        $value = 'final';
    }

    return $value;
}

/** @return 'final' */
function overwriteDefined(): string
{
    $value = 1;

    try {
        echo 'try';
    } finally {
        $value = 'final';
    }

    return $value;
}

/** @return 'final' */
function defineInFinally(): string
{
    try {
        echo 'try';
    } finally {
        $value = 'final';
    }

    return $value;
}

/** @return 'left'|'right' */
function assignInBothBranches(bool $assign, bool $left): string
{
    if ($assign) {
        $value = 1;
    }

    try {
        echo 'try';
    } finally {
        if ($left) {
            $value = 'left';
        } else {
            $value = 'right';
        }
    }

    return $value;
}

function conditionalAssignment(bool $assign): string
{
    try {
        echo 'try';
    } finally {
        if ($assign) {
            $value = 'final';
        }
    }

    // @mago-expect analysis:possibly-undefined-variable
    return $value;
}
