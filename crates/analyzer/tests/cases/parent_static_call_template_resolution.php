<?php

class FooPSCTR
{
    public string $value = 'hello';
}

/**
 * @template TContext of array
 */
class BasePSCTR
{
    /**
     * @param \Closure(TContext): mixed $callback
     */
    public static function run(\Closure $callback): void
    {
    }
}

/**
 * @extends BasePSCTR<array{foo: FooPSCTR}>
 */
class ChildPSCTR extends BasePSCTR
{
    public static function execute(): void
    {
        parent::run(
            fn (array $context) => $context['foo']->value,
        );
    }
}
