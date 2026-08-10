<?php

declare(strict_types=1);

trait PrivateMethodTrait
{
    private function method(): void {}
}

class ParentClass
{
    use PrivateMethodTrait;
}

class ChildClass extends ParentClass
{
    private function method(): void {}
}

trait PublicizedPrivateMethodTrait
{
    private function publicizedMethod(): void {}
}

class PublicizedParentClass
{
    use PublicizedPrivateMethodTrait {
        publicizedMethod as public;
    }
}

class PublicizedChildClass extends PublicizedParentClass
{
    // @mago-expect analysis:missing-override-attribute
    public function publicizedMethod(): void {}
}
