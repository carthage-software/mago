<?php

class UnsortedClass
{
    // Methods in random order - should stay as-is when sorting is disabled
    private function zMethod(): void
    {
    }

    public static function aStaticMethod(): void
    {
    }

    public function __construct()
    {
    }

    protected function mMethod(): void
    {
    }

    public function bMethod(): void
    {
    }
}
