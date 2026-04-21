<?php

declare(strict_types=1);

namespace Foo;

json_encode('a');

// @mago-expect analysis:incorrect-function-casing
Json_Encode('b');
