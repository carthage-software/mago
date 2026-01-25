<?php

declare(strict_types=1);

class UnusedPrivate
{
    // @mago-expect analysis:unused-property
    private string $unused = '';
}

class UsedPrivate
{
    private string $used = '';

    public function get(): string
    {
        return $this->used;
    }
}

class UnderscorePrefix
{
    private string $_intentionallyUnused = '';
}

final class FinalWithProtected
{
    // @mago-expect analysis:unused-property
    protected string $unused = '';
}

class NonFinalWithProtected
{
    protected string $maybeUsedByChild = '';
}

class PublicProperty
{
    public string $unused = '';
}
