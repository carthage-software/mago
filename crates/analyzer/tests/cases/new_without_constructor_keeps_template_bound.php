<?php

declare(strict_types=1);

/**
 * @template T of int
 */
final class TemplateBoundWithoutConstructor
{
    /**
     * @param T $value
     */
    public function add($value): void
    {
        echo $value;
    }
}

function template_bound_without_constructor(): void
{
    $box = new TemplateBoundWithoutConstructor();
    $box->add(1);
    /** @mago-expect analysis:invalid-argument */
    $box->add('a');
}
