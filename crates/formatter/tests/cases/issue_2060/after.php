<?php

class Foo
{
    public function isActive(): bool
    {
        return $this->checkConditionA()
            && $this->checkConditionB();
    }

    public function defaulted(): mixed
    {
        $result = $this->relation?->property
            ?? new DefaultValue();

        return $result;
    }

    public function findOrFail(int $id): Model
    {
        return (
            $this->find(id: $id)
                ?? throw new NotFoundException($id)
        );
    }
}
