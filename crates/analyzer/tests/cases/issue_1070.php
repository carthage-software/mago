<?php

class Foo
{
    protected int $bar = 42;

    protected static int $baz = 24;

    public function getBaz(): int
    {
        return self::$baz;
    }

    public static function setBaz(int $value): void
    {
        self::$baz = $value;
    }

    public function getBar(): int
    {
        return $this->bar;
    }

    public function setBar(int $value): void
    {
        $this->bar = $value;
    }
}

interface Bar
{
    public int $qux {
        get;
        set;
    }

    public function getQux(): int;

    public function setQux(int $value): void;
}

/**
 * @psalm-require-extends Foo
 * @require-implements Bar
 */
trait Baz
{
    public function process(): void
    {
        $bar = $this->getBar();
        $this->setBar($bar + 1);

        $baz = self::$baz;
        self::$baz = $baz + 1;
    }

    public function processInterface(): void
    {
        $qux = $this->getQux();
        $this->setQux($qux + 1);

        $qux = $this->qux;
        $this->qux = $qux + 1;
    }

    public function getBar(): int
    {
        return $this->bar;
    }
}
