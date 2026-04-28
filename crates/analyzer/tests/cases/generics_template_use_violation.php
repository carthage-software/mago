<?php

declare(strict_types=1);

/**
 * @template T of int
 */
trait GenIntTraitOnly
{
    /** @var T */
    public mixed $val;

    /** @param T $v */
    public function __construct(mixed $v)
    {
        $this->val = $v;
    }
}

/**
 * @mago-expect analysis:invalid-template-parameter
 */
final class GenStrUse
{
    /**
     * @use GenIntTraitOnly<string>
     */
    use GenIntTraitOnly;
}
