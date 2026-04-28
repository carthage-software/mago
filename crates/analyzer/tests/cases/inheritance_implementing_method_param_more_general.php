<?php

declare(strict_types=1);

class InhAnimalGen
{
}

class InhDogGen extends InhAnimalGen
{
}

interface InhParamGenIface
{
    public function feed(InhDogGen $a): void;
}

class InhParamGenImpl implements InhParamGenIface
{
    public function feed(InhAnimalGen $a): void
    {
    }
}

(new InhParamGenImpl())->feed(new InhDogGen());
