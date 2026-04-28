<?php

declare(strict_types=1);

class InhPromoSuperParent
{
    public function __construct(public string $name)
    {
    }

    public function display(): string
    {
        return $this->name;
    }
}

class InhPromoSuperChild extends InhPromoSuperParent
{
    public function __construct(string $name, public int $age)
    {
        parent::__construct($name);
    }
}

$c = new InhPromoSuperChild('a', 30);
echo $c->display();
echo $c->age;
