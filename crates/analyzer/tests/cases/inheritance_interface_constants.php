<?php

declare(strict_types=1);

interface InhIfaceConst
{
    public const string GREETING = 'hello';
}

class InhIfaceConstImpl implements InhIfaceConst
{
}

echo InhIfaceConst::GREETING;
echo InhIfaceConstImpl::GREETING;
