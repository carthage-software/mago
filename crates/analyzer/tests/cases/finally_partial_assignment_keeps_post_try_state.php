<?php

declare(strict_types=1);

function finally_partial_g(): void {}

function finally_partial_assignment(): int
{
    $t = null;
    try {
        $t = 1;
        finally_partial_g();
    } finally {
        if (rand(0, 1)) {
            $t = 2;
        }
    }

    return $t;
}

function finally_partial_assignment_after_catch(): string
{
    $s = null;
    try {
        $s = 'a';
        finally_partial_g();
    } catch (Exception) {
        $s = 'b';
    } finally {
        if (rand(0, 1)) {
            $s = 'c';
        }
    }

    return $s;
}

function finally_partial_assignment_with_return_in_try(): int
{
    $t = 0;
    try {
        if (rand(0, 1)) {
            return 5;
        }

        $t = 1;
        finally_partial_g();
    } finally {
        if (rand(0, 1)) {
            $t = 2;
        }
    }

    return $t + 1;
}

function finally_partial_assignment_keeps_new_type(): void
{
    $t = null;
    try {
        finally_partial_g();
    } finally {
        if (rand(0, 1)) {
            $t = new Exception();
        }
    }

    if ($t !== null) {
        echo $t->getMessage();
    }
}

function finally_partial_assignment_in_nested_catch(): void
{
    $t = null;
    try {
        finally_partial_g();
    } finally {
        try {
            finally_partial_g();
        } catch (Throwable $e) {
            $t = $e;
        }
    }

    if ($t !== null) {
        echo $t->getMessage();
    }
}

function finally_partial_assignment_in_loop(): void
{
    $t = null;
    try {
        finally_partial_g();
    } finally {
        foreach ([1, 2] as $i) {
            if ($i > rand(0, 3)) {
                $t = $i;
            }
        }
    }

    if ($t !== null) {
        echo $t;
    }
}

function finally_full_assignment(): int
{
    $t = null;
    try {
        finally_partial_g();
    } finally {
        $t = 1;
    }

    return $t;
}

function finally_partial_unset(): int
{
    $t = 1;
    try {
        finally_partial_g();
    } finally {
        if (rand(0, 1)) {
            unset($t);
        }
    }

    return $t;
}

function finally_partial_assignment_after_possibly_undefined_catch(): void
{
    try {
        $x = rand(0, 1);
    } catch (RuntimeException) {
        echo 'caught';
    } finally {
        if (rand(0, 1)) {
            $x = 2;
        }
    }

    echo $x;
}

function finally_partial_assignment_only(): void
{
    try {
        finally_partial_g();
    } finally {
        if (rand(0, 1)) {
            $u = 1;
        }
    }

    echo $u;
}
