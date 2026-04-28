<?php

declare(strict_types=1);

class InhCtorParent
{
    public function __construct(public int $value)
    {
    }
}

class InhCtorChild extends InhCtorParent
{
    public function __construct(int $value, public string $name)
    {
        parent::__construct($value);
    }
}

$c = new InhCtorChild(1, 'a');
echo $c->value;
echo $c->name;
