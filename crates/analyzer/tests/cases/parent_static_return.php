<?php

/**
 * Test case for parent::method() with static return type.
 *
 * When a child class method calls parent::method() and returns it,
 * PHP's late static binding means the `static` return type should
 * resolve to the child class, not the parent class.
 *
 * @see https://github.com/vimeo/psalm/issues/11139
 */

interface AI {
    /** @return $this */
    public function test(): static;
}

class A implements AI {
    public function test(): static {
        return $this;
    }
}

class B extends A {
    public function test(): static {
        return parent::test();
    }
}

// More complex case with multiple levels of inheritance
class C extends B {
    public function test(): static {
        return parent::test();
    }
}

// Case without interface
class D {
    public function method(): static {
        return $this;
    }
}

class E extends D {
    public function method(): static {
        return parent::method();
    }
}
