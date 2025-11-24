<?php

declare(strict_types=1);

class Foo
{
}

class ChildFoo extends Foo
{
    public function a(): void
    {
    }
}

// Test 1: Child has more specific native type than parent docblock (issue #657)
// The child's native return type (ChildFoo) should be used, not inherited from parent's docblock (Foo)
abstract class Root
{
    /**
     * @return Foo
     */
    abstract public function getFoo();
}

class Child extends Root
{
    public function getFoo(): ChildFoo
    {
        return new ChildFoo();
    }

    public function work(): void
    {
        // Should work - getFoo returns ChildFoo, which has method a()
        $this->getFoo()->a();
    }
}

// Test 2: Parent docblock is more specific than child native type
// The parent's docblock type (ChildFoo) should be inherited because it's more specific
abstract class Root2
{
    /**
     * @return ChildFoo
     */
    abstract public function getBar();
}

class Child2 extends Root2
{
    public function getBar(): Foo
    {
        return new ChildFoo();
    }

    public function work(): void
    {
        // Should work - inherited ChildFoo from parent docblock
        $this->getBar()->a();
    }
}

// Test 3: Both have same specificity - child native type should be used
abstract class Root3
{
    /**
     * @return ChildFoo
     */
    abstract public function getBaz();
}

class Child3 extends Root3
{
    public function getBaz(): ChildFoo
    {
        return new ChildFoo();
    }

    public function work(): void
    {
        // Should work - both types are ChildFoo
        $this->getBaz()->a();
    }
}

// Test 4: Parent has no docblock, child has native type
abstract class Root4
{
    abstract public function getQux();
}

class Child4 extends Root4
{
    public function getQux(): ChildFoo
    {
        return new ChildFoo();
    }

    public function work(): void
    {
        // Should work - child's native type is used
        $this->getQux()->a();
    }
}
