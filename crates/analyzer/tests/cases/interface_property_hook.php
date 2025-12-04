<?php

interface MigratesUp
{
    public string $name {
        get;
    }
}

// This should work - class provides a hook implementation
class Migration implements MigratesUp
{
    public string $name {
        get => 'migration';
    }
}

// This should also work - class provides a regular property (which implicitly has get)
class Migration2 implements MigratesUp
{
    public string $name = 'migration2';
}
