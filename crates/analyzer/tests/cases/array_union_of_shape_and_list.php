<?php

declare(strict_types=1);

/** @param array{style: string, ...} $options */
function render(array $options): string
{
    return $options['style'];
}

/** @param list<string> $classes */
function widget(array $classes): string
{
    return render(['style' => 's'] + $classes);
}

/** @param list<string> $classes */
function widget_reversed(array $classes): string
{
    return render($classes + ['style' => 's']);
}

/**
 * @param list{int, string} $sealed
 * @return array{0: int, 1: true}
 */
function overrides_an_item(array $sealed): array
{
    return [1 => true] + $sealed;
}

/**
 * @param list{int, string} $sealed
 * @return array{0: int, 1: string, a: int}
 */
function adds_a_key(array $sealed): array
{
    return ['a' => 1] + $sealed;
}
