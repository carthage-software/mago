<?php

declare(strict_types=1);

class Constraint {}

interface ValidatorInterface
{
    /**
     * @param Constraint|Constraint[] $constraints
     */
    public function validate(mixed $value, Constraint|array|null $constraints = null): array;
}

function run(ValidatorInterface $validator): void
{
    $validator->validate('foo', null);
}
