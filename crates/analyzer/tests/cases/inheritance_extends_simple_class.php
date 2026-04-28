<?php

declare(strict_types=1);

class InhSimpleParent
{
    public function greet(): string
    {
        return 'hi';
    }
}

class InhSimpleChild extends InhSimpleParent
{
}

$child = new InhSimpleChild();
echo $child->greet();
