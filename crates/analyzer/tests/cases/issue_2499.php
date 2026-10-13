<?php

declare(strict_types=1);

namespace Issue2499;

final class A
{
    public function __construct(public bool $b, public ?string $s) {}
}

final class Probe
{
    public function reproOr(): bool
    {
        $messages = [];
        $flag = false;

        foreach ([new A(false, null), new A(true, 'm')] as $a) {
            $flag = $flag || $a->b;
            if ($a->s !== null) {
                $messages[] = $a->s;
            }
        }

        if ($flag) {
            return true;
        }

        return false;
    }

    public function reproAnd(): bool
    {
        $messages = [];
        $flag = true;

        foreach ([new A(false, null), new A(true, 'm')] as $a) {
            $flag = $flag && $a->b;
            if ($a->s !== null) {
                $messages[] = $a->s;
            }
        }

        if (!$flag) {
            return true;
        }

        return false;
    }

    public function reversedOr(): bool
    {
        $messages = [];
        $flag = false;

        foreach ([new A(false, null), new A(true, 'm')] as $a) {
            $flag = $a->b || $flag;
            if ($a->s !== null) {
                $messages[] = $a->s;
            }
        }

        if ($flag) {
            return true;
        }

        return false;
    }
}

function outsideLoop(bool $flag, bool $right, ?string $message): bool
{
    $messages = [];
    $flag = $flag || $right;

    if ($message !== null) {
        $messages[] = $message;
    }

    if ($flag) {
        return true;
    }

    return false;
}

function booleanComparison(bool $flag, ?string $message): bool
{
    $messages = [];
    $flag = $flag === false;

    if ($message !== null) {
        $messages[] = $message;
    }

    if ($flag) {
        return true;
    }

    return false;
}

function whileLoop(A $a, bool $repeat): bool
{
    $messages = [];
    $flag = false;

    while ($repeat) {
        $flag = $flag || $a->b;
        if ($a->s !== null) {
            $messages[] = $a->s;
        }

        $repeat = (bool) rand(0, 1);
    }

    if ($flag) {
        return true;
    }

    return false;
}
