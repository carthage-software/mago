<?php

declare(strict_types=1);

class InhRetUnrA
{
}

class InhRetUnrB
{
}

interface InhRetUnrIface
{
    public function build(): InhRetUnrA;
}

class InhRetUnrImpl implements InhRetUnrIface
{
    /** @mago-expect analysis:incompatible-return-type */
    public function build(): InhRetUnrB
    {
        return new InhRetUnrB();
    }
}
