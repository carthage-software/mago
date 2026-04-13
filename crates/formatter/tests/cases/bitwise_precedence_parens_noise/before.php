<?php

$comparisonShiftWithoutGrouping = $mask === $flags << 1;
$comparisonShiftWithGrouping = $mask === ($flags << 1);
$coalesceShiftWithoutGrouping = $value ?? $flags >> 1;
$coalesceShiftWithGrouping = $value ?? ($flags >> 1);

$mixedBitwiseWithoutGrouping = $a & $b | $c;
$mixedBitwiseWithGrouping = ($a & $b) | $c;
$mixedXorWithoutGrouping = $a & $b ^ $c;
$mixedXorWithGrouping = ($a & $b) ^ $c;
$mixedShiftWithoutGrouping = $a << $b & $c;
$mixedShiftWithGrouping = ($a << $b) & $c;
$mixedShiftSamePrecedenceWithoutGrouping = $a << $b >> $c;
$mixedShiftSamePrecedenceWithGrouping = ($a << $b) >> $c;

$requiredComparisonGrouping = ($a & $b) === $c;
$requiredMixedBitwiseGrouping = ($a | $b) & $c;
$requiredMixedXorGrouping = ($a ^ $b) & $c;
$requiredRightHandGrouping = $a & ($b | $c);
$requiredShiftGrouping = ($a & $b) << $c;
$arithmeticChildGrouping = $a ^ ($b + $c);
