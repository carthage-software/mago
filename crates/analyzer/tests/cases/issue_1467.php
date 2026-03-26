<?php

// ok
setcookie('foo', 'bar', 0, '/');

// ok
setcookie('foo', 'bar', ['expires' => 0]);

// @mago-expect analysis:too-many-arguments - too many args
setcookie('foo', 'bar', ['expires' => 0], '/');
