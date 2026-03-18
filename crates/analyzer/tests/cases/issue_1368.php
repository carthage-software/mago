<?php

declare(strict_types=1);

abstract class AbstractModel1368
{
    public function __construct(public int $id) {}
}

class ModelA1368 extends AbstractModel1368 {}

class Service1368
{
    /**
     * @template T of AbstractModel1368
     * @param T ...$models
     * @return array<T>
     */
    private function sortModels(AbstractModel1368 ...$models): array
    {
        return $models;
    }

    /** @return array<ModelA1368> */
    public function test(): array
    {
        $list = [new ModelA1368(1), new ModelA1368(2)];
        return $this->sortModels(...$list);
    }
}
