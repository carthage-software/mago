<?php

declare(strict_types=1);

interface Probe {}

/**
 * @param class-string $class
 */
function narrows_through_a_dynamic_class_argument(object|string $value, string $class): void
{
    if (is_a($value, $class, true)) {
        accepts_object_or_class_string($value);
    }

    if (is_subclass_of($value, $class, false)) {
        accepts_object($value);
    }
}

/**
 * @param class-string<Probe> $class
 */
function keeps_the_template_of_a_dynamic_class_argument(object|string $value, string $class): void
{
    if (is_a($value, $class, true)) {
        accepts_probe_or_probe_class_string($value);
    }
}

function accepts_object(object $value): void
{
    echo $value::class;
}

/**
 * @param object|class-string $value
 */
function accepts_object_or_class_string(object|string $value): void
{
    echo is_object($value) ? $value::class : $value;
}

/**
 * @param Probe|class-string<Probe> $value
 */
function accepts_probe_or_probe_class_string(Probe|string $value): void
{
    echo is_object($value) ? $value::class : $value;
}
