<?php

declare(strict_types=1);

namespace App;

class Foo
{
    public function printSomething(bool $bool): void
    {
        if ($bool) {
            $bar = new Bar()
                ->setTest1('test1')
                ->setTest2('test2')
            ;

            echo $bar->getTest1();
        } else {
            $bar = new Bar()
                ->setTest1('test1')
                ->setTest2('test2')
            ;

            echo $bar->getTest2();
        }
    }

    public function other(int $b): void
    {
        if ($b === 1) {
            $x = new Bar()
                ->setTest1('a')
                ->setTest2('b')
            ;
        } elseif ($b === 2) {
            $x = new Bar()->setTest1('c');
        } else {
            $x = new Bar()->setTest2('d');
        }
    }
}
