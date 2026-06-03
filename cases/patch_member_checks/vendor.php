<?php

abstract class VendorClass
{
    final const DISALLOW_REMOVING_FINAL = 1;
    private const DISALLOW_LOOSENING_VISIBILITY = 2;
    const ALLOW_ADDING_FINAL = 3;
    const ALLOW_ADDING_TYPE = 4;

    public function allowMarkingAbstract(): void {}
    final public function disallowRemovingFinal(): void {}
    public function disallowAddingStatic(): void {}
    public static function disallowRemovingStatic(): void {}
    protected function disallowLooseningVisibility(): void {}
    public function allowAddingFinal(): void {}
    public function allowNarrowingParameterType(mixed $value): void {}

    final public mixed $disallowRemovingFinal;
    public mixed $disallowAddingReadonly;
    public mixed $disallowAddingHooks;
    public mixed $disallowAddingStatic;
    public static mixed $disallowRemovingStatic;
    private mixed $disallowLooseningVisibility;
    public mixed $allowAddingFinal;
    public mixed $allowAddingType;
}
