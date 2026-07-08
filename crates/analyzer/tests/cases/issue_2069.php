<?php

declare(strict_types=1);

/** @require-extends SomeThing */
trait TheTrait
{
    public function doATraitThing(): void
    {
        $this->doSomething();
    }
}

class SomeThing
{
    use TheTrait;

    public function doSomething(): void {}
}

class OtherThing extends SomeThing
{
    use TheTrait;
}
