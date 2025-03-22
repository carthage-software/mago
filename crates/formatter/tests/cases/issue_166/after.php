<?php

final class Issue166FirstCase
{
    public function __construct(#[Eager] public BWithEager $b) {}

    #[ConsoleCommand]
    public function test(
        #[ConsoleArgument]
        null|int $optionalValue = null,
        #[ConsoleArgument(name: 'custom-flag')]
        bool $flag = false,
    ): void {
    }

    #[ConsoleCommand]
    public function test2(#[ConsoleArgument] null|int $optionalValue = null): void
    {
    }
}

final class Issue166SecondCase
{
    public function __construct(
        #[Eager]
        public BWithEager $b,
    ) {}

    #[ConsoleCommand]
    public function test(
        #[ConsoleArgument]
        null|int $optionalValue = null,
        #[ConsoleArgument(name: 'custom-flag')]
        bool $flag = false,
    ): void {
    }

    #[ConsoleCommand]
    public function test2(
        #[ConsoleArgument]
        null|int $optionalValue = null,
    ): void {
    }
}
