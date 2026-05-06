<?php

declare(strict_types=1);

class InhAnimal
{
    public function clone1(): InhAnimal
    {
        return new self();
    }
}

class InhDog extends InhAnimal
{
    #[\Override]
    public function clone1(): InhDog
    {
        return new self();
    }
}

(new InhDog())->clone1();
