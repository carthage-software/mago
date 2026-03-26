<?php

declare(strict_types=1);

interface Textable1355
{
    public function text(): string;
}

class Model1355 {}

/**
 * @template T of Textable1355
 */
readonly class ViewTable1355
{
    /**
     * @param array<T> $models
     */
    public function __construct(
        public array $models,
    ) {}
}

/**
 * @mago-expect analysis:template-constraint-violation
 * @mago-expect analysis:possibly-invalid-argument
 */
$table = new ViewTable1355(models: [new Model1355()]);
