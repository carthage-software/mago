<?php

declare(strict_types=1);

class InvalidOverrideBase
{
    public function doSomeThing(): void {}
}

class InvalidOverrideKid extends InvalidOverrideBase
{
    // @mago-expect analysis:invalid-override-attribute
    #[Override]
    public function notThere(): void {}

    #[Override]
    public function doSomeThing(): void {}
}
