<?php

ResponseFactory::instance()
    ->fail([
        'message' => 'Some very very very long message that exceeds the configured max line length bla bla bla bla',
    ], 400)
    ->respond();

$result = SomeFactory::create()->process(['a', 'b', 'c'])->finalize();
