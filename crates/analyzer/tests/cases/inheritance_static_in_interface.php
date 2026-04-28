<?php

declare(strict_types=1);

interface InhStaticIface
{
    public static function build(): self;
}

class InhStaticIfaceImpl implements InhStaticIface
{
    public static function build(): self
    {
        return new self();
    }
}

InhStaticIfaceImpl::build();
