<?php

declare(strict_types=1);

/** @return null|Closure */
$get_closure = function (string|callable $str): ?Closure {
    if (is_string($str)) {
        if (function_exists($str)) {
            return Closure::fromCallable($str);
        }

        return null;
    }

    return Closure::fromCallable($str);
};

/** @return null|Closure */
$get_closure = function (string|callable $str): ?Closure {
    if (is_string($str)) {
        if (is_callable($str)) {
            return Closure::fromCallable($str);
        }

        return null;
    }

    return Closure::fromCallable($str);
};

/** @return null|Closure */
$get_closure = function (string|callable $str): ?Closure {
    if (is_callable($str)) {
        return Closure::fromCallable($str);
    }

    return null;
};
