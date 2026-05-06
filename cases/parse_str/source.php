<?php

declare(strict_types=1);

$query_string = 'list[]=1&list[]=2&list[]=3&arr[k1]=val&arr[k2]=val&foo=bar&one=1';

parse_str($query_string, $out);

$result = array_key_exists('arr', $out) && is_array($out['arr']) || is_string($out['arr']);
