<?php

class MyClass
{
    /**
     * @mago-expect analysis:assignment-to-this
     */
    public function reassignThis(): void
    {
        $this = new MyClass();
    }
}
