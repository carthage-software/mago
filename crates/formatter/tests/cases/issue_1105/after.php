<?php

$object
    ->method1()
    ->method2()
    ->method3()
;

$object->method1()->method2();

$object
    ->method1()
    ->method2()
    ->method3()
;

$this
    ->getCache()
    ->forget()
;

$object
    ->methodWithALongName('some argument')
    ->anotherMethodWithALongName('another argument')
    ->yetAnotherMethodWithALongName('yet another argument')
;

foo(
    $this
        ->getCache()
        ->forget(),
);
