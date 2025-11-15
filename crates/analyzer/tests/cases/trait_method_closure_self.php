<?php

declare(strict_types=1);

trait FooTrait
{
    /**
     * @param non-empty-array<string, \Closure(array<array-key, mixed>, self):array<array-key, mixed>> $steps
     */
    protected function applySteps(array $steps): void
    {
    }
}

class Foo
{
    use FooTrait;

    public function test(): void
    {
        $this->applySteps([
            'step1' => function (array $data, self $_context): array {
                return $data;
            },
            'step2' => function (array $data, Foo $_context): array {
                return $data;
            },
        ]);
    }
}
