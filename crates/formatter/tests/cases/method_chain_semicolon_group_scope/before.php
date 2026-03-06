<?php

$value = $builder->create()
    ->stepOne()
    ->stepTwo()
;

$other = some_call(
    $argOne,
    $argTwo,
);

$result = $factory->make()
    ->withFoo()
    ->withBar()
;

$wrapped = wrap(
    $builder->create()
        ->stepOne()
        ->stepTwo(),
);

return $response->build()
    ->withStatus(200)
    ->withPayload($value)
;
