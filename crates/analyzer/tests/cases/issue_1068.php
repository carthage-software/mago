<?php

declare(strict_types=1);

class ProtoEnumType
{
    const CASE_A = 1;
    const CASE_B = 2;
}

enum EnumType: int
{
    case CaseA = ProtoEnumType::CASE_A;
    case CaseB = ProtoEnumType::CASE_B;
}

$array = [
    EnumType::CaseA->value => 'CASE_A',
];

foreach ($array as $key => $value) {
    echo "Key: $key, Value: $value\n";
}
