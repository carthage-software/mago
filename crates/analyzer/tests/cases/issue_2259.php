<?php

declare(strict_types=1);

namespace Issue2259;

/** @param -3|0|22 $_ */
function acceptIntegerKey(int $_): void
{
}

/** @param '+2'|'-0'|'01'|'word' $_ */
function acceptStringKey(string $_): void
{
}

$keys = ['22', '0', '-3'];
$dependencyList = [];

foreach ($keys as $key) {
    $dependencyList[$key] = 2342;
}

foreach ($dependencyList as $dependon => $colList) {
    acceptIntegerKey($dependon);
    $key = (string) $dependon;
}

$keys = ['01', '-0', '+2', 'word'];
$dependencyList = [];

foreach ($keys as $key) {
    $dependencyList[$key] = 2342;
}

foreach ($dependencyList as $dependon => $colList) {
    acceptStringKey($dependon);
}
