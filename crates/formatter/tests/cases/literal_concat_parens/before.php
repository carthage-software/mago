<?php

$a = (1) . (2);
$b = (2) . 'foo';
$c = ($a + 2) . 'foo';
$d = 'foo' . ($a + 2);

$a = (1.0) . (2.0);
$b = (2.0) . 'foo';
$c = ($a + 2.0) . 'foo';
$d = 'foo' . ($a + 2.0);
