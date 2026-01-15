<?php

declare(strict_types=1);

namespace X\Z {
    class ExistingClass2 {}
}

namespace X {
    class ExistingClass {}

    function existingFunction(Z\ExistingClass2 $_existingClass): void
    {
    }
}

namespace Y {
    use X\ExistingClass;

    function existingFunction(ExistingClass $_existingClass): void
    {
    }

    interface TestInterface {}
}

namespace {
    use Y\TestInterface;

    class A
    {
        public TestInterface $testInterface = null;
        /** @mago-expect analysis: non-existent-class-like */
        public ?X $x = null;

        /** @mago-expect analysis: non-existent-class-like */
        public X $x2 = null;

        public function __construct(/** @mago-expect analysis: non-existent-class-like */ B $_x) {}
    }

    function foo(/** @mago-expect analysis: non-existent-class-like */ Z $_z): void
    {
    }

    function bar(/** @mago-expect analysis: non-existent-class-like */ ?Z $_z): void
    {
    }

    function z(\X\ExistingClass $_existingClass): void
    {
    }
}

