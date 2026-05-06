<?php

/**
 * @param string $x
 */
function param_mismatch(int $x): void
{
    echo $x;
}

/**
 * @return int
 */
function return_mismatch(): string
{
    return 1;
}

/** @param int $x */
function param_drops_native_string(int|string $x): void
{
    echo $x;
}

/** @param non-empty-string $x */
function param_narrower(string $x): void
{
    echo $x;
}

/** @return non-empty-string */
function return_narrower(): string
{
    return 'hello';
}

/** @param positive-int $x */
function param_narrower_int(int $x): void
{
    echo $x;
}

class Foo
{
    /**
     * @param string $x
     */
    public function method_param_mismatch(int $x): void
    {
        echo $x;
    }

    /**
     * @return int
     */
    public function method_return_mismatch(): string
    {
        return 1;
    }
}

$closure_param_mismatch =
    /** @param string $x */
    static function (int $x): void {
        echo $x;
    };

$closure_return_mismatch =
    /** @return int */
    static function (): string {
        return 1;
    };

$arrow_return_mismatch =
    /** @return int */
    static fn(): string => 1;
