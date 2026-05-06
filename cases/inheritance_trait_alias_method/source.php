<?php

declare(strict_types=1);

trait InhAliasA
{
    public function pick(): string
    {
        return 'A';
    }
}

trait InhAliasB
{
    public function pick(): string
    {
        return 'B';
    }
}

class InhAliasUser
{
    use InhAliasA, InhAliasB {
        InhAliasA::pick insteadof InhAliasB;
        InhAliasB::pick as pickB;
    }
}

$u = new InhAliasUser();
echo $u->pick();
echo $u->pickB();
