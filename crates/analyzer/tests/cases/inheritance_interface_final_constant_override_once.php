<?php

declare(strict_types=1);

interface InhIfaceFConstOnce
{
    final public const int X = 1;
}

interface InhIfaceFConstOnceMiddle extends InhIfaceFConstOnce
{
}

interface InhIfaceFConstOnceInner extends InhIfaceFConstOnceMiddle
{
}

class InhIfaceFConstOnceParent implements InhIfaceFConstOnce
{
}

class InhIfaceFConstOnceChild extends InhIfaceFConstOnceParent
{
    public const int X = 2;
}

class InhIfaceFConstOnceReimplementing extends InhIfaceFConstOnceParent implements InhIfaceFConstOnceMiddle
{
    public const int X = 2;
}

class InhIfaceFConstOnceIndirect implements InhIfaceFConstOnceMiddle
{
    public const int X = 2;
}

class InhIfaceFConstOnceBoth implements InhIfaceFConstOnce, InhIfaceFConstOnceMiddle
{
    public const int X = 2;
}

class InhIfaceFConstOnceDeep implements InhIfaceFConstOnceInner
{
    public const int X = 2;
}
