<?php

declare(strict_types=1);

/**
 * @param class-string $class
 */
function takes_class_string(string $class): void
{
    echo $class;
}

/**
 * @param interface-string $interface
 */
function takes_interface_string(string $interface): void
{
    echo $interface;
}

function rejects_non_strings(int $i, bool $b): void
{
    /** @mago-expect analysis:invalid-argument */
    takes_class_string($i);

    /** @mago-expect analysis:invalid-argument */
    takes_class_string($b);
}

function coerces_plain_string(string $s): void
{
    /** @mago-expect analysis:possibly-invalid-argument */
    takes_class_string($s);
}

function accepts_class_strings(): void
{
    takes_class_string(Throwable::class);
    takes_class_string(Exception::class);
    takes_interface_string(Throwable::class);
}

/**
 * @param class-string $class
 * @param class-string<Throwable> $throwable
 */
function accepts_narrower_class_strings(string $class, string $throwable): void
{
    takes_class_string($class);
    takes_class_string($throwable);
}
