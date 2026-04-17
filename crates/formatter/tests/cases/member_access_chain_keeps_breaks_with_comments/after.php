<?php

$data = Writer::default()
    ->u8(1) // version
    ->u16(42) // type
    ->toString();

$res = $client
    ->db // connection
    ->stmt // prepared statement
    ->run();

$ok = $x->a->b->c();
