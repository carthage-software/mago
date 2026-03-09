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
