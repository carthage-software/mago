<?php

declare(strict_types=1);

interface InhIfaceFConst
{
    final public const int X = 1;
}

class InhIfaceFConstClass implements InhIfaceFConst
{
    /** @mago-expect analysis:override-final-constant */
    public const int X = 2;
}

abstract class InhIfaceFConstAbstract implements InhIfaceFConst
{
    /** @mago-expect analysis:override-final-constant */
    public const int X = 2;
}

final class InhIfaceFConstFinal implements InhIfaceFConst
{
    /** @mago-expect analysis:override-final-constant */
    final public const int X = 2;
}

interface InhIfaceFConstExtending extends InhIfaceFConst
{
    /** @mago-expect analysis:override-final-constant */
    public const int X = 2;
}

enum InhIfaceFConstEnum implements InhIfaceFConst
{
    /** @mago-expect analysis:override-final-constant */
    public const int X = 2;
}

interface InhIfaceFConstMiddle extends InhIfaceFConst
{
}

class InhIfaceFConstIndirect implements InhIfaceFConstMiddle
{
    /** @mago-expect analysis:override-final-constant */
    public const int X = 2;
}

class InhIfaceFConstParent implements InhIfaceFConst
{
}

class InhIfaceFConstChild extends InhIfaceFConstParent
{
    /** @mago-expect analysis:override-final-constant */
    public const int X = 2;
}

class InhIfaceFConstReimplementing extends InhIfaceFConstParent implements InhIfaceFConst
{
    /** @mago-expect analysis:override-final-constant */
    public const int X = 2;
}

class InhIfaceFConstPlainParent
{
    public const int X = 1;
}

class InhIfaceFConstPlainChild extends InhIfaceFConstPlainParent implements InhIfaceFConst
{
    /** @mago-expect analysis:override-final-constant */
    public const int X = 2;
}

$anonymous = new class implements InhIfaceFConst {
    /** @mago-expect analysis:override-final-constant */
    public const int X = 2;
};

interface InhIfaceConstOpen
{
    public const int X = 1;
}

class InhIfaceConstOpenOverride implements InhIfaceConstOpen
{
    public const int X = 2;
}

class InhIfaceFConstInherited implements InhIfaceFConst
{
}

class InhIfaceFConstTwice implements InhIfaceFConst, InhIfaceFConstMiddle
{
}

class InhIfaceFConstOtherCase implements InhIfaceFConst
{
    public const int x = 2;
}

/** @require-implements InhIfaceFConstMiddle */
trait InhIfaceFConstRequiring
{
    public const int X = 2;
}

echo InhIfaceConstOpenOverride::X;
echo InhIfaceFConstInherited::X;
echo InhIfaceFConstTwice::X;
echo InhIfaceFConstOtherCase::x;
