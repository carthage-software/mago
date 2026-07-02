<?php

$result = bagOf(array_values($theInputCollection))->containsAll($firstNeedle, $secondNeedle, normalize($thirdNeedleValue ?? $thirdNeedle));
