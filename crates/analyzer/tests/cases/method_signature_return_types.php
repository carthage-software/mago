<?php

// Test 1: Narrowing return type (OK - covariance)
// PHP: No error
abstract class Repository
{
    abstract public function find(int $id): null|object;
}

class UserRepository extends Repository
{
    // OK: Returning more specific type
    public function find(int $id): null|User
    {
        return null;
    }
}

// Test 2: Widening return type (ERROR - violates covariance)
// PHP: "Declaration of BadRepo::get(int $id): mixed must be compatible with Base::get(int $id): ?User"
abstract class Base
{
    abstract public function get(int $id): null|User;
}

class BadRepo extends Base
{
    // @mago-expect analysis:incompatible-return-type
    public function get(int $id): mixed
    {
        return null;
    }
}

// Test 3: Completely different return type (ERROR)
// PHP: "Declaration of C1::process(): int must be compatible with I1::process(): string"
interface I1
{
    public function process(): string;
}

class C1 implements I1
{
    // @mago-expect analysis:incompatible-return-type
    public function process(): int
    {
        return 42;
    }
}

// Test 4: Removing nullable from return (OK - narrowing)
// PHP: No error
trait T1
{
    abstract public function fetch(): null|string;
}

class C2
{
    use T1;

    // OK: Never returns null (narrower than ?string)
    public function fetch(): string
    {
        return 'data';
    }
}

// Test 5: Adding nullable to return (ERROR - widening)
// PHP: "Declaration of C3::load(): ?int must be compatible with T2::load(): int"
trait T2
{
    abstract public function load(): int;
}

class C3
{
    use T2;

    // @mago-expect analysis:incompatible-return-type
    public function load(): null|int
    {
        return null;
    }
}

// Test 6: Union type narrowing (OK)
// PHP: No error
interface I2
{
    public function getValue(): string|int|float;
}

class C4 implements I2
{
    // OK: Returning subset of types
    public function getValue(): string|int
    {
        return 'value';
    }
}

// Test 7: Union type widening (ERROR)
// PHP: "Declaration of C5::getData(): string|int|bool must be compatible with I3::getData(): string|int"
interface I3
{
    public function getData(): string|int;
}

class C5 implements I3
{
    // @mago-expect analysis:incompatible-return-type
    public function getData(): string|int|bool
    {
        return 'data';
    }
}

// Test 8: Void to non-void (ERROR)
// PHP: "Declaration of C6::execute(): int must be compatible with I4::execute(): void"
interface I4
{
    public function execute(): void;
}

class C6 implements I4
{
    // @mago-expect analysis:incompatible-return-type
    public function execute(): int
    {
        return 42;
    }
}

// Test 9: Non-void to void (ERROR)
// PHP: "Declaration of C7::run(): void must be compatible with I5::run(): int"
interface I5
{
    public function run(): int;
}

class C7 implements I5
{
    // @mago-expect analysis:incompatible-return-type
    public function run(): void
    {
    }
}

// Test 10: Never type (OK - narrowest possible)
// PHP: No error (never is bottom type)
interface I6
{
    public function fail(): mixed;
}

class C8 implements I6
{
    // OK: never is compatible with any return type
    public function fail(): never
    {
        exit(1);
    }
}

// Dummy class for tests
class User
{
}
