<?php

declare(strict_types=1);

enum HttpMethod
{
    case Get;
    case Post;
    case Put;
    case Delete;
}

function flow_match_multi_value_arm(HttpMethod $m): string
{
    return match ($m) {
        HttpMethod::Get, HttpMethod::Post => 'safe-ish',
        HttpMethod::Put, HttpMethod::Delete => 'mutating',
    };
}
