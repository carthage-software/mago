<?php

declare(strict_types=1);

/**
 * @template T
 */
abstract class GenContBase2
{
    /** @return T */
    abstract public function get(): mixed;
}

/**
 * @extends GenContBase2<string>
 */
final class GenStrContainer extends GenContBase2
{
    public function get(): string
    {
        return 'hi';
    }
}

function take_string(string $s): void
{
}

take_string((new GenStrContainer())->get());
