<?php

(require_once __DIR__ . '/../bootstrap/app.php')->handleRequest(Request::capture());

(require __DIR__ . '/../bootstrap/app.php')->handleRequest(Request::capture());

(include_once __DIR__ . '/../bootstrap/app.php')->handleRequest(Request::capture());

(include __DIR__ . '/../bootstrap/app.php')->handleRequest(Request::capture());

// Long chain, eligible for chaining
(require 'rector-shared.php')
    ->withPaths([__DIR__ . '/apps', __DIR__ . '/lib', __DIR__ . '/core'])
    ->withSkip([SimplifyUselessVariableRector::class])
    ->withRules([SimplifyUselessVariableRector::class])
    ->withSets([SetList::CODE_QUALITY, SetList::DEAD_CODE])
    ->withTypeCoverageLevel(2);

$a = (include 'file.php') + $x;
$b = (require_once 'file.php') . $x;
$c = (print 'hello') && $x;
$d = (include 'file.php') ? 'yes' : 'no';
$e = $x + (include 'file.php');
$f = isset($x) && $y;
$g = include 'file.php';
$h = (include 'file.php') |> strtoupper(...);
