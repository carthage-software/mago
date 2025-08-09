<?php

#[Deprecated]
const FOO = 'foo';

#[ATTRIBUTE]
function aaa(
    int $bbb,
    // this comment causes attribute to disappear
    int $ccc,
) {
    var_dump('test');
}

#[Route(
    'route/path',
    name: 'very_very_very_very_very_very_long_route_name',
    methods: ['GET'],
)]
class Foo
{
}
