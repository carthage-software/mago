<?php

#[Route(?, name: ?)]
class Foo {}

#[Mix('positional', ?, named: ?, ...)]
class Bar {}

#[VeryLongAttributeName(
    'the first positional argument value here',
    ?,
    secondNamedPlaceholder: ?,
    thirdNamedPlaceholder: ?,
    fourth: ?,
)]
class Baz {}
