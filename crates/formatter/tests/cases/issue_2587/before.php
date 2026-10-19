<?php

class A
{
    public const MAP = ['x' => B::class];

    public function make(string $k): object
    {
        return new (self::MAP[$k])();
    }
}

new (A::MAP[$k])();
new (A::MAP[$k]['class'])();
new (A::MAP[$k]->class)();
new (A::MAP[$k]?->class)();
new (A::MAP[$k]::$class)();
new (A::class)();
new (CLASS_MAP[$k])();
new (TARGET_CLASS)();
new ('stdClass')();
new (['stdClass'][$k])();
new (get_classes()[$k])();
new ($holder->get_classes()[$k])();

new ($classes[$k])();
new ($classes[$k]['class'])();
new ($holder->classes[$k])();
new ($holder?->classes[$k])();
new (A::$classes[$k])();
new ($holder::$classes[$k])();
new ($class)();
new A();
