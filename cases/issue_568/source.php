<?php

/** @var array{foo?: array{bar?: string, baz: string}} $y */
$y = ['foo' => ['baz' => 123]];
$x = null;

/**
 */
echo $y['foo']['baz'];

/**
 */
echo $x['foo'];
