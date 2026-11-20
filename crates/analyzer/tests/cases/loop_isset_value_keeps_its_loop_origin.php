<?php

declare(strict_types=1);

final class LoopIssetBox
{
    public null|LoopIssetBox $next = null;

    public function merge(LoopIssetBox $other): LoopIssetBox
    {
        return $other;
    }
}

/**
 * @param array<int, LoopIssetBox> $values
 *
 * @return array<int, LoopIssetBox>
 */
function loop_isset_method_call(array $values): array
{
    $merged = [];
    foreach ($values as $key => $value) {
        if (isset($merged[$key])) {
            $merged[$key] = $merged[$key]->merge($value);
        } else {
            $merged[$key] = $value;
        }
    }

    return $merged;
}

/**
 * @param array<int, LoopIssetBox> $values
 *
 * @return array<int, LoopIssetBox|null>
 */
function loop_isset_property_fetch(array $values): array
{
    $nexts = [];
    foreach ($values as $key => $value) {
        if (isset($nexts[$key])) {
            $nexts[$key] = $nexts[$key]->next;
        } else {
            $nexts[$key] = $value;
        }
    }

    return $nexts;
}
