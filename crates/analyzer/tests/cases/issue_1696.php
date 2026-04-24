<?php

declare(strict_types=1);

/**
 * @return non-empty-string
 */
function reassign_ternary(mixed $param): string
{
    $is_ok = is_string($param) && ($param = trim($param)) !== '';

    if ($is_ok) {
        echo $param;
    }

    return $is_ok ? $param : 'fallback';
}

/**
 * @return non-empty-string
 */
function reassign_early_return(mixed $param): string
{
    $is_ok = is_string($param) && ($param = trim($param)) !== '';

    if ($is_ok) {
        return $param;
    }

    return 'fallback';
}

/**
 * @return non-empty-string
 */
function reassign_if_else_early_return(mixed $param): string
{
    if (is_string($param) && ($param = trim($param)) !== '') {
        $is_ok = true;
    } else {
        $is_ok = false;
    }

    if ($is_ok) {
        // Tracking a branch-discriminator boolean across if/else bodies is not supported, so we don't
        // know that $param is a string here.
        //
        // @mago-expect analysis:mixed-return-statement
        return $param;
    }

    return 'fallback';
}

/**
 * @return non-empty-string
 */
function no_reassign_ternary(mixed $param): string
{
    $is_ok = is_string($param) && $param !== '';

    return $is_ok ? $param : 'fallback';
}
