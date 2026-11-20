<?php

declare(strict_types=1);

interface ConditionalClosureFactory
{
}

/**
 * @return ($value is Closure ? 'closure' : 'other')
 */
function conditional_closure_kind(mixed $value): string
{
    return $value instanceof Closure ? 'closure' : 'other';
}

/**
 * @return 'other'
 */
function conditional_closure_on_non_closures(string $text, Countable $countable, ConditionalClosureFactory $factory): string
{
    conditional_closure_kind($countable);
    conditional_closure_kind($factory);

    return conditional_closure_kind($text);
}

/**
 * @return 'closure'
 */
function conditional_closure_on_closure(Closure $closure): string
{
    return conditional_closure_kind($closure);
}
