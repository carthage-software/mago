<?php

declare(strict_types=1);

class UnusedPrivateMethod
{
    // @mago-expect analysis:unused-method
    private function unused(): void
    {
    }
}

class UsedPrivateMethod
{
    private function helper(): void
    {
    }

    public function main(): void
    {
        $this->helper();
    }
}

class UnderscoreMethod
{
    private function _intentionallyUnused(): void
    {
    }
}

final class FinalWithProtectedMethod
{
    // @mago-expect analysis:unused-method
    protected function unused(): void
    {
    }
}

class NonFinalWithProtectedMethod
{
    protected function maybeUsedByChild(): void
    {
    }
}

class PublicMethod
{
    public function unused(): void
    {
    }
}
