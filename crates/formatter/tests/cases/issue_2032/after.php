<?php

$result = bagOf(array_values($theInputCollection))->containsAll(
    $firstNeedle,
    $secondNeedle,
    $thirdNeedleValue ?? $thirdNeedle,
);
