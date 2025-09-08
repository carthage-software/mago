<?php

declare(strict_types=1);

/**
 * @mago-expect analysis:missing-return-statement
 */
function x(bool $x): int
{
    if ($x === true) {
        return 15;
    }
}

/**
 * @mago-expect analysis:missing-return-statement
 */
function y(bool $x): int
{
}

function i_go_in_circles(): int
{
    do {
        if (rand(0, 1) === 1) {
            return 15;
        }
    } while (true);
}

function i_go_in_circles2(): int
{
    while (true) {
        if (rand(0, 1) === 1) {
            return 15;
        }
    }
}

function i_go_in_circles3(): int
{
    for (;;) {
        if (rand(0, 1) === 1) {
            return 15;
        }
    }
}

function i_go_in_circles4(): int
{
    while (true) {
        if (rand(0, 1) === 1) {
            return 15;
        } else {
            continue;
        }
    }
}

function i_go_in_circles_and_never_return(): int
{
    while (true) {
        if (rand(0, 1) === 1) {
            // do nothing
        }
    }
}

function i_go_in_circles_and_never_return2(): int
{
    do {
        if (rand(0, 1) === 1) {
            // do nothing
        }
    } while (true);
}

function i_go_in_circles_and_never_return3(): int
{
    for (;;) {
        if (rand(0, 1) === 1) {
            // do nothing
        }
    }
}

function i_go_in_circles_and_never_return4(): int
{
    while (true) {
        if (rand(0, 1) === 1) {
            // do nothing
        } else {
            continue;
        }
    }
}

/**
 * @mago-expect analysis:missing-return-statement
 */
function i_might_not_go_in_circles(): int
{
    while (true) {
        if (rand(0, 1) === 1) {
            return 15;
        } else {
            break;
        }
    }
}

/**
 * @mago-expect analysis:missing-return-statement
 */
function i_might_not_go_in_circles2(): int
{
    do {
        if (rand(0, 1) === 1) {
            return 15;
        } else {
            break;
        }
    } while (true);
}
