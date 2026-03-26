<?php

declare(strict_types=1);

trait HelperTrait
{
    protected static function helperMethod(): string
    {
        return 'hello';
    }
}

abstract class BaseClass
{
    use HelperTrait;

    protected static function ownMethod(): string
    {
        return 'world';
    }
}

/** @require-extends BaseClass */
trait MyTrait
{
    public function doStuff(): string
    {
        $a = self::ownMethod();
        $b = self::helperMethod();

        return $a . $b;
    }
}

final class ConcreteClass extends BaseClass
{
    use MyTrait;
}

// Also test with deeper trait nesting
trait DeepHelperTrait
{
    protected static function deepMethod(): string
    {
        return 'deep';
    }
}

trait MiddleTrait
{
    use DeepHelperTrait;

    protected static function middleMethod(): string
    {
        return 'middle';
    }
}

abstract class MiddleBaseClass
{
    use MiddleTrait;
}

/** @require-extends MiddleBaseClass */
trait DeepTrait
{
    public function doDeepStuff(): string
    {
        // Methods from traits used by traits used by the required class
        return self::deepMethod() . self::middleMethod();
    }
}

final class DeepConcreteClass extends MiddleBaseClass
{
    use DeepTrait;
}
