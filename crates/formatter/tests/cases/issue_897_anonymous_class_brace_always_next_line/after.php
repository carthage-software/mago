<?php

declare(strict_types=1);

$singleLine = new class($a, $b)
{
    public function foo(): void
    {
    }
};

$multiLineArgs = new class(
    $veryLongArgumentNameThatExceedsTheLineWidth,
    $anotherVeryLongArgumentNameForTesting,
    $yetAnotherExtremelyLongArgumentName,
)
{
    public function foo(): void
    {
    }
};

$withExtends = new class extends SomeVeryLongBaseClassName implements
    SomeVeryLongInterfaceName,
    AnotherVeryLongInterfaceName,
    YetAnotherExtremelyLongInterfaceName
{
    public function foo(): void
    {
    }
};
