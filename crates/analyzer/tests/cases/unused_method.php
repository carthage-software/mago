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

class UsedViaFirstClassCallable
{
    private function helper(string $name): string
    {
        return "Hello, $name";
    }

    public function main(): void
    {
        $closure = $this->helper(...);
        $closure("World");
    }
}

class UsedViaPartialApplication
{
    private function greet(string $greeting, string $name): string
    {
        return "$greeting, $name";
    }

    public function main(): void
    {
        $sayHello = $this->greet("Hello", ?);
        $sayHello("World");
    }
}

class UsedStaticViaFirstClassCallable
{
    private static function helper(int $x): int
    {
        return $x * 2;
    }

    public function main(): void
    {
        $closure = self::helper(...);
        $closure(5);
    }
}

class UsedStaticViaPartialApplication
{
    private static function add(int $a, int $b): int
    {
        return $a + $b;
    }

    public function main(): void
    {
        $addFive = self::add(5, ?);
        $addFive(10);
    }
}
