<?php

declare(strict_types=1);

interface InheritedTraitSignatureContract
{
    public function be(string $user): void;
}

trait InheritedTraitSignatureTrait
{
    /** @mago-expect analysis:incompatible-return-type */
    public function be(string $user): static
    {
        return $this;
    }
}

abstract class InheritedTraitSignatureBase implements InheritedTraitSignatureContract
{
    use InheritedTraitSignatureTrait;
}

final class InheritedTraitSignatureFirstChild extends InheritedTraitSignatureBase
{
}

final class InheritedTraitSignatureSecondChild extends InheritedTraitSignatureBase
{
}
