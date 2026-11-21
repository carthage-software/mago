<?php

declare(strict_types=1);

interface ParentClassAndInterfaceProvider
{
    /**
     * @return int
     */
    public function lifetime();
}

abstract class ParentClassAndInterfaceBase
{
    /**
     * @return int|bool|null
     */
    protected function lifetime()
    {
        return null;
    }
}

final class ParentClassAndInterfaceChild extends ParentClassAndInterfaceBase implements ParentClassAndInterfaceProvider
{
    public function lifetime()
    {
        return 0;
    }
}
