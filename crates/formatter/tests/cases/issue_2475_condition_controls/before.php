<?php

if ($firstConditionWithAnExtraLongVariableName || ($secondConditionWithAnExtraLongVariableName && $thirdConditionWithAnExtraLongVariableName)) {
    echo 'if';
} elseif ($firstConditionWithAnExtraLongVariableName && ($secondConditionWithAnExtraLongVariableName || $thirdConditionWithAnExtraLongVariableName)) {
    echo 'elseif';
}

while ($firstConditionWithAnExtraLongVariableName && ($secondConditionWithAnExtraLongVariableName || $thirdConditionWithAnExtraLongVariableName)) {
    break;
}

do {
    echo 'do';
} while ($firstConditionWithAnExtraLongVariableName || ($secondConditionWithAnExtraLongVariableName && $thirdConditionWithAnExtraLongVariableName));

switch ($firstConditionWithAnExtraLongVariableName || ($secondConditionWithAnExtraLongVariableName && $thirdConditionWithAnExtraLongVariableName)) {
    case true:
        break;
    default:
        break;
}

$result = match ($firstConditionWithAnExtraLongVariableName && ($secondConditionWithAnExtraLongVariableName || $thirdConditionWithAnExtraLongVariableName)) {
    true => 'yes',
    false => 'no',
};

if ($a || ($b && ($c || $d))) {
    echo 'short';
}
