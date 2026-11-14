<?php

declare(strict_types=1);

/**
 * @template T of object
 *
 * @psalm-type Pair = array{0: T, 1: T}
 */
final class Data
{
    /** @var Pair|null */
    public ?array $pair = null;
}
