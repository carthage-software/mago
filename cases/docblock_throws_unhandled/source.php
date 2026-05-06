<?php

declare(strict_types=1);

final class BoomF extends RuntimeException {}

/**
 * @throws BoomF
 */
function will_throw(): void
{
    throw new BoomF();
}

/**
 */
function unhandled_caller(): void
{
    will_throw();
}

try {
    unhandled_caller();
} catch (BoomF) {
}
