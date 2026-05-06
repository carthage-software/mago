<?php

// ok
setcookie('foo', 'bar', 0, '/');

// ok
setcookie('foo', 'bar', ['expires' => 0]);

setcookie('foo', 'bar', ['expires' => 0], '/');
