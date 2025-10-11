<?php

declare(strict_types=1);

trait TraitWithPrivateMethod {
    private static function privateInTraitLevel2(): void {
        echo __METHOD__, PHP_EOL;
    }
}
trait TraitWithTrait {
    use TraitWithPrivateMethod;
    private static function privateInTraitLevel1(): void {
        echo __METHOD__, PHP_EOL;
    }
}

final class SomeClass {
    use TraitWithTrait;

    public function __construct()
    {
        self::privateInTraitLevel2();
        self::privateInTraitLevel1();
    }
}

$nonEmptyList = new SomeClass();
