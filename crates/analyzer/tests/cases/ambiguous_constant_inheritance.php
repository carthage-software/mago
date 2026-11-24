<?php

declare(strict_types=1);

interface AmbA
{
    public const X = 1;
}

interface AmbB
{
    public const X = 1;
}

class AmbTwoInterfaces implements AmbA, AmbB {}

class AmbParent
{
    public const X = 2;
}

class AmbParentAndInterface extends AmbParent implements AmbA {}

class AmbGrandchild extends AmbParent {}

class AmbGrandparentAndInterface extends AmbGrandchild implements AmbA {}

interface AmbExtendsBoth extends AmbA, AmbB {}

enum AmbEnum implements AmbA, AmbB {}

interface AmbChildOverride extends AmbA
{
    public const X = 3;
}

class AmbChildAndParentInterface implements AmbChildOverride, AmbA {}

class AmbOverridingParent implements AmbA
{
    public const X = 4;
}

class AmbReimplements extends AmbOverridingParent implements AmbA {}

trait AmbTrait
{
    public const X = 5;
}

class AmbTraitParent
{
    use AmbTrait;
}

class AmbTraitParentAndInterface extends AmbTraitParent implements AmbA {}

class AmbOwnOverride implements AmbA, AmbB
{
    public const X = 6;
}

interface AmbOwnOverrideInterface extends AmbA, AmbB
{
    public const X = 7;
}

class AmbOwnTrait implements AmbA
{
    use AmbTrait;
}

class AmbPrivateParent
{
    private const X = 8;
}

class AmbPrivateParentAndInterface extends AmbPrivateParent implements AmbA {}

interface AmbNode
{
    public const KEY = 'key';
}

interface AmbField extends AmbNode {}

interface AmbRoot extends AmbNode {}

interface AmbPagination extends AmbRoot {}

abstract class AmbFieldBase implements AmbField {}

abstract class AmbPaginationBase extends AmbFieldBase implements AmbPagination {}

class AmbSameDeclaration implements AmbField, AmbRoot {}
