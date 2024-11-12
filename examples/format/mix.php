<?php

declare(ticks=1);
declare(encoding="UTF-8");

namespace App\Demo;

use some\space;

use function some\other\space\funct;

use const some\other\space\CONST;

const MYCONST = 123;

const a = 1,
    c = 3,
    d = 4,
    c = 3,
    d = 4,
    c = 3,
    d = 4,
    c = 3,
    d = 4,
    c = 3,
    d = 4,
    c = 3,
    d = 4,
    c = 3,
    d = 4,
    c = 3,
    d = 4,
    c = 3,
    d = 4,
    c = 3,
    d = 4;

if ($a) {
    echo "a";
} else if ($b) {
    echo "b";
} else {
    echo "c";
}

do {
    echo "a";
}while ($a);

$a = 12;

if ($b);

function bar(): baz&(null|baz)
{
    return "bar";
}

trait A
{
    private function smallTalk()
    {
        echo 'a';
    }
    private function bigTalk()
    {
        echo 'A';
    }
}

trait B
{
    private function smallTalk()
    {
        echo 'b';
    }
    private function bigTalk()
    {
        echo 'B';
    }
}

trait C
{
    public function smallTalk()
    {
        echo 'c';
    }
    public function bigTalk()
    {
        echo 'C';
    }
}

class Talker
{
    use A, B, C {
        B::smallTalk as public;
        A::bigTalk as public;

        B::smallTalk insteadof A, C;
        A::bigTalk insteadof B, C;

        B::bigTalk as public Btalk;
        A::smallTalk as public asmalltalk;

        C::bigTalk as Ctalk;
        C::smallTalk as cmallstalk;
    }
}

(new Talker())->bigTalk();
(new Talker())->Btalk();
(new Talker())->Ctalk();
(new Talker())->asmalltalk();
(new Talker())->smallTalk();
(new Talker())->cmallstalk();
