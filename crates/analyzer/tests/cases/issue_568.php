<?php

/** @var array{foo?: array{bar?: string, baz: string}} $y */
$y = ['foo' => ['baz' => 123]];
$x = null;

/**
 * @mago-expect analysis:possibly-undefined-string-array-index
 * @mago-expect analysis:possibly-null-array-access
 */
echo $y['foo']['baz'];

/**
 * @mago-expect analysis:null-array-access
 */
echo $x['foo'];
