<?php

// Test 1: Direct trait constant access (ERROR)
trait Foo
{
    const A = 42;
}

Foo::A;

// Test 2: Access through class (OK)
class Bar
{
    use Foo;
}

echo Bar::A; // No error - accessing through class

// Test 3: Multiple constants from same trait
trait Config
{
    const HOST = 'localhost';
    const PORT = 8080;
    const DEBUG = true;
}

Config::HOST;

Config::PORT;

Config::DEBUG;

// Test 4: Access through class is fine
class Database
{
    use Config;
}

echo Database::HOST; // OK
echo Database::PORT; // OK
$debug = Database::DEBUG; // OK

// Test 5: Trait::class is ALLOWED in PHP (special case)
trait MyTrait {}

$class = MyTrait::class; // OK - ::class is allowed on traits

// Test 6: Nested trait usage
trait BaseTrait
{
    const VALUE = 'base';
}

trait MiddleTrait
{
    use BaseTrait;
}

MiddleTrait::VALUE;

// But through class is OK
class UsingMiddle
{
    use MiddleTrait;
}

echo UsingMiddle::VALUE; // OK

// Test 7: In statement context (not assignments to avoid cascading errors)
trait Settings
{
    const TIMEOUT = 30;
}

Settings::TIMEOUT;

// Test 8: Another direct access
trait Math
{
    const PI = 3.14159;
}

Math::PI;
