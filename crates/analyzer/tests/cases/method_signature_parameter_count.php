<?php

// Test 1: Child requires MORE required params than parent (ERROR)
// PHP: "Declaration of C1::foo(int $a, int $b, int $c) must be compatible with T1::foo(int $a, int $b)"
trait T1
{
    abstract public function foo(int $a, int $b);
}

class C1
{
    use T1;

    // @mago-expect analysis:incompatible-parameter-count
    public function foo(int $a, int $b, int $c)
    {
    }
}

// Test 2: Child has MORE params but they're optional (OK)
// PHP: No error
trait T2
{
    abstract public function bar(int $a, int $b);
}

class C2
{
    use T2;

    // OK: Extra optional parameter is allowed
    public function bar(int $a, int $b, int $c = 3)
    {
    }
}

// Test 3: Child has FEWER required params than parent requires (ERROR)
// PHP: "Declaration of C3::baz(int $a) must be compatible with T3::baz(int $a, int $b)"
trait T3
{
    abstract public function baz(int $a, int $b);
}

class C3
{
    use T3;

    // @mago-expect analysis:incompatible-parameter-count
    public function baz(int $a)
    {
    }
}

// Test 4: Parent has optional params, child omits them (ERROR)
// PHP: "Declaration of C4::process(string $data) must be compatible with I1::process(string $data, array $options = [])"
interface I1
{
    public function process(string $data, array $options = []);
}

class C4 implements I1
{
    // @mago-expect analysis:incompatible-parameter-count
    public function process(string $data)
    {
    }
}

// Test 5: Child adds optional params to parent with optional params (OK)
// PHP: No error
interface I2
{
    public function transform(string $input, bool $strict = false);
}

class C5 implements I2
{
    // OK: Adding more optional params
    public function transform(string $input, bool $strict = false, int $timeout = 30)
    {
    }
}

// Test 6: Making parent's optional param required (ERROR)
// PHP: "Declaration of C6::validate(string $data, array $rules) must be compatible with I3::validate(string $data, array $rules = [])"
interface I3
{
    public function validate(string $data, array $rules = []);
}

class C6 implements I3
{
    // @mago-expect analysis:incompatible-parameter-count
    public function validate(string $data, array $rules)
    {
    }
}

// Test 7: No params in parent, child adds required params (ERROR)
// PHP: "Declaration of C7::execute(string $cmd) must be compatible with I4::execute()"
interface I4
{
    public function execute();
}

class C7 implements I4
{
    // @mago-expect analysis:incompatible-parameter-count
    public function execute(string $cmd)
    {
    }
}

// Test 8: No params in parent, child adds optional params (OK)
// PHP: No error
interface I5
{
    public function run();
}

class C8 implements I5
{
    // OK: Adding optional params to parameterless method
    public function run(bool $async = false)
    {
    }
}

// Test 9: Interface extends interface, adds required params (ERROR)
// PHP: "Declaration of I7::save(object $entity, bool $flush) must be compatible with I6::save(object $entity)"
interface I6
{
    public function save(object $entity);
}

interface I7 extends I6
{
    // @mago-expect analysis:incompatible-parameter-count
    public function save(object $entity, bool $flush);
}

// Test 10: Interface extends interface, adds optional params (OK)
// PHP: No error
// NOTE: Analyzer does not check interface-to-interface signatures (limitation)
interface I8
{
    public function delete(int $id);
}

interface I9 extends I8
{
    public function delete(int $id, bool $soft = true);
}
