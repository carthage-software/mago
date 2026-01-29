<?php

declare(strict_types=1);

$singleLine = new class($a, $b) {
    public function foo(): void
    {
    }
};

$multiLineArgs = new class(
    $veryLongArgumentNameThatExceedsTheLineWidth,
    $anotherVeryLongArgumentNameForTesting,
    $yetAnotherExtremelyLongArgumentName,
) {
    public function foo(): void
    {
    }
};
