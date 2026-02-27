<?php

array_merge($map, (@include base_path('file.php')) ?: []);

$a = (@include base_path('file.php')) ?: [];

$b = (@require base_path('file.php')) ?: [];

$c = (@include_once base_path('file.php')) ?: [];

$d = (@require_once base_path('file.php')) ?: [];

$e = (@include base_path('file.php')) ? true : false;
