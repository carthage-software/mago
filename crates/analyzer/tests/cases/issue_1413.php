<?php

declare(strict_types=1);

trait RestrictionTrait
{
    /**
     * @throws LogicException
     */
    public function assertNotRestricted(string $type): int
    {
        if (!method_exists($this, 'getUser')) {
            throw new LogicException('not defined');
        }

        /** @var string $user */
        $user = $this->getUser();

        return $this->doSomething($user);
    }

    private function doSomething(string $v): int
    {
        return strlen($v);
    }
}
