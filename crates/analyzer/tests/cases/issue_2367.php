<?php

declare(strict_types=1);

namespace Issue2367;

enum ParameterType
{
    case NULL;
    case INTEGER;
}

final class Where
{
    /** @var int[]|ParameterType[] */
    public array $params = [];
}

$where = new Where();

/** @var int[]|ParameterType[] $arr */
$arr = require 'a.php';
$where->params = $arr;

/** @var array<int|ParameterType> $arr */
$arr = require 'a.php';
$where->params = $arr;

/** @var array<array-key, int|ParameterType> $arr */
$arr = require 'a.php';
$where->params = $arr;

/** @var array<array-key, int|ParameterType::NULL|ParameterType::INTEGER> $arr */
$arr = require 'a.php';
$where->params = $arr;
