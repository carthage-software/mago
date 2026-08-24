<?php

namespace test;

$profilingResults = [
    ['Duration' => '2'],
];

$totalTime = (float) array_sum(array_column($profilingResults, 'Duration'));
