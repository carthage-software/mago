<?php

declare(strict_types=1);

// ok: int form with all args
session_set_cookie_params(3600, '/', '.example.com', true, true);

// ok: int form with just lifetime
session_set_cookie_params(3600);

// ok: array form with 1 arg
session_set_cookie_params(['lifetime' => 3600, 'path' => '/']);

// @mago-expect analysis:too-many-arguments - array form with extra args
session_set_cookie_params(['lifetime' => 3600], '/');
