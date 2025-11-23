<?php

trait T {
    public const CONSTANT = 4;
    public const ANOTHER = "hello";

    public function doSomething(): void {
        echo T::CONSTANT; // @mago-expect analysis:direct-trait-constant-access

        echo self::CONSTANT;
        echo static::CONSTANT;
        echo $this::CONSTANT;

        echo self::ANOTHER;
        echo static::ANOTHER;
        echo $this::ANOTHER;
    }
}

class S {
    use T;
}

class U {
    use T;

    public function test(): void {
        echo self::CONSTANT;
        echo static::CONSTANT;
        echo $this::CONSTANT;
    }
}

$s = new S;
$s->doSomething();

$u = new U;
$u->test();

echo S::CONSTANT;
echo U::CONSTANT;
