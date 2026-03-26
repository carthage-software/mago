<?php

// Test 1: Trait method overrides final parent method (ERROR)
class Base1 {
    final public function test() {
        return 4;
    }
}

trait Trait1 {
    public function test() {
        return 3;
    }
}

// @mago-expect analysis:override-final-method
class Child1 extends Base1 {
    use Trait1;
}

// Test 2: Trait with alias overrides final parent method (SKIP - Issue 52)
// This requires codex changes to process trait adaptations
// class Base2 {
//     final public function foo() {
//         return 1;
//     }
// }
//
// trait Trait2 {
//     public function bar() {
//         return 2;
//     }
// }
//
// class Child2 extends Base2 {
//     // @mago-expect analysis:override-final-method
//     use Trait2 { bar as foo; }
// }

// Test 3: Multiple trait levels - final parent method (ERROR)
class Base3 {
    final public function calculate() {
        return 0;
    }
}

trait MiddleTrait {
    public function calculate() {
        return 5;
    }
}

trait TopTrait {
    use MiddleTrait;
}

// @mago-expect analysis:override-final-method
class Child3 extends Base3 {
    use TopTrait;
}

// Test 4: Non-final parent method (OK)
class Base4 {
    public function test() {
        return 1;
    }
}

trait Trait4 {
    public function test() {
        return 2;
    }
}

class Child4 extends Base4 {
    use Trait4;  // OK - parent method is not final
}

// Test 5: Trait method doesn't override parent (OK)
class Base5 {
    final public function finalMethod() {
        return 1;
    }
}

trait Trait5 {
    public function differentMethod() {
        return 2;
    }
}

class Child5 extends Base5 {
    use Trait5;  // OK - trait method has different name
}

// Test 6: Final method from grandparent (ERROR)
class GrandParent {
    final public function ancestorMethod() {
        return 100;
    }
}

class Parent6 extends GrandParent {
}

trait Trait6 {
    public function ancestorMethod() {
        return 200;
    }
}

// @mago-expect analysis:override-final-method
class Child6 extends Parent6 {
    use Trait6;
}

// Test 7: Protected final method (ERROR)
class Base7 {
    final protected function protectedFinal() {
        return 'protected';
    }
}

trait Trait7 {
    protected function protectedFinal() {
        return 'trait';
    }
}

// @mago-expect analysis:override-final-method
class Child7 extends Base7 {
    use Trait7;
}

// Test 8: Private final method from parent (OK - different scope)
class Base8 {
    final private function _privateFinal() {
        return 'base';
    }
}

trait Trait8 {
    private function _privateFinal() {
        return 'trait';
    }
}

class Child8 extends Base8 {
    use Trait8;  // OK - private methods are not inherited
}
