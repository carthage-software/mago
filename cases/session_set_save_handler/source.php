<?php

declare(strict_types=1);

// ok: object form with 1 arg
session_set_save_handler(new SessionHandler());

// ok: object form with 2 args
session_set_save_handler(new SessionHandler(), false);

// ok: callable form with 6 args
session_set_save_handler(fn() => true, fn() => true, fn() => '', fn() => true, fn() => true, fn() => 0);

// ok: callable form with optional args
session_set_save_handler(
    fn() => true,
    fn() => true,
    fn() => '',
    fn() => true,
    fn() => true,
    fn() => 0,
    fn() => '',
    fn() => true,
    fn() => true,
);

session_set_save_handler(new SessionHandler(), false, fn() => true);

session_set_save_handler(fn() => true, fn() => true);
