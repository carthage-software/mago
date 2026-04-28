<?php

declare(strict_types=1);

final class ClassesPrivateSameClassAccess
{
    public function __construct(private int $value)
    {
    }

    public function compare(self $other): bool
    {
        return $this->value === $other->value;
    }
}

$a = new ClassesPrivateSameClassAccess(1);
$b = new ClassesPrivateSameClassAccess(2);
echo $a->compare($b) ? 'eq' : 'neq';
