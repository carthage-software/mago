<?php

class UninitializedInConstructor
{
    // @mago-expect analysis:uninitialized-property
    public string $name;
    public int $age;

    public function __construct()
    {
        $this->age = 25;

        // $this->name is not initialized!
    }
}

class AllInitialized
{
    public string $name;
    public int $age;

    public function __construct()
    {
        $this->name = 'test';
        $this->age = 25;
    }
}

class InitializedViaPrivateMethod
{
    public string $name;

    public function __construct()
    {
        $this->init();
    }

    private function init(): void
    {
        $this->name = 'test';
    }
}

class InitializedViaFinalMethod
{
    public string $name;

    public function __construct()
    {
        $this->init();
    }

    final protected function init(): void
    {
        $this->name = 'test';
    }
}

class NotInitializedViaProtectedMethod
{
    // @mago-expect analysis:uninitialized-property
    public string $name;

    public function __construct()
    {
        $this->init(); // Can be overridden in child class!
    }

    protected function init(): void
    {
        $this->name = 'test';
    }
}

final class FinalClassProtectedOk
{
    public string $name;

    public function __construct()
    {
        $this->init(); // OK - final class
    }

    protected function init(): void
    {
        $this->name = 'test';
    }
}

class TransitiveInitialization
{
    public string $name;

    public function __construct()
    {
        $this->helper();
    }

    private function helper(): void
    {
        $this->doInit();
    }

    private function doInit(): void
    {
        $this->name = 'test';
    }
}
