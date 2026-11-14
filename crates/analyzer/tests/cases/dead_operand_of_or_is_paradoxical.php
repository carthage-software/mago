<?php

declare(strict_types=1);

final class Model
{
    public const string A = 'MODEL_A';
    public const string B = 'MODEL_B';
}

function dead_on_the_left(string $m, bool $flag): int
{
    if ($m !== Model::A) {
        /** @mago-expect analysis:paradoxical-condition */
        if ($m === Model::A || $flag) {
            return 1;
        }
    }

    return 0;
}

function dead_on_the_right(string $m, bool $flag): int
{
    if ($m !== Model::A && $m !== Model::B) {
        /** @mago-expect analysis:paradoxical-condition */
        if ($flag || $m === Model::A) {
            return 1;
        }
    }

    return 0;
}

function dead_in_a_chain(string $m, bool $flag, bool $other): int
{
    if ($m !== Model::A) {
        /** @mago-expect analysis:paradoxical-condition */
        if ($flag || $m === Model::A || $other) {
            return 1;
        }
    }

    return 0;
}

function alternatives_of_one_variable(string $m): int
{
    if ($m !== Model::A) {
        if ($m === Model::A || $m === Model::B) {
            return 1;
        }
    }

    return 0;
}

function live_operands(string $m, bool $flag): int
{
    if ($m !== Model::A) {
        if ($flag || $m === Model::B) {
            return 1;
        }
    }

    return 0;
}
