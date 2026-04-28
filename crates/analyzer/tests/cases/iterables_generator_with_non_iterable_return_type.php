<?php

declare(strict_types=1);

/**
 * @mago-expect analysis:invalid-generator-return-type
 */
function bad_generator_return(): int
{
    yield 1;
    return 1;
}

/**
 * @mago-expect analysis:invalid-generator-return-type
 */
function bad_generator_void(): void
{
    yield 1;
}

/**
 * @mago-expect analysis:invalid-generator-return-type
 */
function bad_generator_string(): string
{
    yield 'x';
    return 'done';
}
