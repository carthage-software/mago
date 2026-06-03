<?php

abstract class VendorClass
{
    const int DISALLOW_REMOVING_FINAL = 1;
    public const DISALLOW_LOOSENING_VISIBILITY = 2;
    final const int ALLOW_ADDING_FINAL = 3;
    const int ALLOW_ADDING_TYPE = 4;

    abstract public function allowMarkingAbstract(): void;
    public function disallowRemovingFinal(): void {}
    public static function disallowAddingStatic(): void {}
    public function disallowRemovingStatic(): void {}
    public function disallowLooseningVisibility(): void {}
    final public function allowAddingFinal(): void {}
    public function allowNarrowingParameterType(string $value): void {}

    public mixed $disallowRemovingFinal;
    public readonly mixed $disallowAddingReadonly;
    public mixed $disallowAddingHooks {
        get => null;
    }
    public static mixed $disallowAddingStatic;
    public mixed $disallowRemovingStatic;
    public mixed $disallowLooseningVisibility;
    final public mixed $allowAddingFinal;
    public string $allowAddingType;
}
