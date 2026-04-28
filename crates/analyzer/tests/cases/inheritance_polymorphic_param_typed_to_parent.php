<?php

declare(strict_types=1);

class InhPolyAnimal
{
    public function name(): string
    {
        return 'animal';
    }
}

class InhPolyDog extends InhPolyAnimal
{
    #[\Override]
    public function name(): string
    {
        return 'dog';
    }
}

function inh_describe(InhPolyAnimal $a): string {
    return $a->name();
}

echo inh_describe(new InhPolyDog());
echo inh_describe(new InhPolyAnimal());
