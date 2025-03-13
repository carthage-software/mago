<?php
echo 'Hello, world!';
declare(strict_types=1);
namespace MyNamespace;
use My\ClassA;
use My\ClassB;
class MyClass
{
    const MY_CONST = 1;
    public int $myProperty;
    public function myMethod()
    {
        if (true) {
        }
        for ($i = 0; $i < 10; $i++) {
        }
        foreach ([1, 2, 3] as $item) {
        }
        while (false) {
        }
        do {
        } while (false);
        switch (1) {
            case 1:
                break;
        }
        return 1;
        // This is a dangling comment
    }
    public int $mySecondProperty;
    const MY_SECOND_CONST = 2;
    use MyTrait;
}
enum MyEnum: int
{
    case CASE_A = 1;
    case CASE_B = 2;
}
trait MyTrait
{
}
namespace Example {
    class MyOtherClass
    {
    }
}
namespace ExampleTwo {
    class MyGlobalClass
    {
    }
}