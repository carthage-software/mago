<?php

// Test 1: Narrowing parameter type (ERROR - violates contravariance)
// PHP: "Declaration of C1::log(string $message) must be compatible with I1::log(string|int $message)"
interface I1 {
    public function log(string|int $message);
}

class C1 implements I1 {
    // @mago-expect analysis:incompatible-parameter-type
    public function log(string $message) {}
}

// Test 2: Widening parameter type (OK - contravariance allows this)
// PHP: No error
interface I2 {
    public function process(string $data);
}

class C2 implements I2 {
    // OK: Can accept more types than parent
    public function process(string|int $data) {}
}

// Test 3: Changing to incompatible type (ERROR)
// PHP: "Declaration of C3::handle(int $value) must be compatible with I3::handle(string $value)"
interface I3 {
    public function handle(string $value);
}

class C3 implements I3 {
    // @mago-expect analysis:incompatible-parameter-type
    public function handle(int $value) {}
}

// Test 4: Widening to mixed (OK)
// PHP: No error
trait T1 {
    abstract public function accept(int $x);
}

class C4 {
    use T1;

    // OK: mixed accepts everything including int
    public function accept(mixed $x) {}
}

// Test 5: Narrowing from mixed (ERROR)
// PHP: "Declaration of C5::receive(string $x) must be compatible with T2::receive(mixed $x)"
trait T2 {
    abstract public function receive(mixed $x);
}

class C5 {
    use T2;

    // @mago-expect analysis:incompatible-parameter-type
    public function receive(string $x) {}
}

// Test 6: Object type widening (OK)
// PHP: No error
abstract class Base {
    abstract public function save(User $entity);
}

class Repository extends Base {
    // OK: object is wider than User
    public function save(object $entity) {}
}

// Test 7: Object type narrowing (ERROR)
// PHP: "Declaration of Repo2::persist(User $entity) must be compatible with Base2::persist(object $entity)"
abstract class Base2 {
    abstract public function persist(object $entity);
}

class Repo2 extends Base2 {
    // @mago-expect analysis:incompatible-parameter-type
    public function persist(User $entity) {}
}

// Test 8: Nullable to non-nullable narrowing (ERROR)
// PHP: "Declaration of C6::find(int $id) must be compatible with I4::find(?int $id)"
interface I4 {
    public function find(?int $id);
}

class C6 implements I4 {
    // @mago-expect analysis:incompatible-parameter-type
    public function find(int $id) {}
}

// Test 9: Non-nullable to nullable widening (OK)
// PHP: No error
interface I5 {
    public function search(string $term);
}

class C7 implements I5 {
    // OK: Accepts null in addition to string
    public function search(?string $term) {}
}

// Test 10: Array shape narrowing (ERROR)
// PHP: "Declaration of C8::configure(array $config) must be compatible with I6::configure(array $config)"
// Note: PHP doesn't validate array shapes at runtime, but types differ
interface I6 {
    /**
     * @param array<string, mixed> $config
     */
    public function configure(array $config);
}

class C8 implements I6 {
    /**
     * @param array<string, string> $config
     * @mago-expect analysis:incompatible-parameter-type
     */
    public function configure(array $config) {}
}

// Dummy class for tests
class User {}
