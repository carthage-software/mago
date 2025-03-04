<?php

class Example {
    public function something(): void {
        $command = array_find(
            array: $this->consoleConfig->commands,
            callback: fn (ConsoleCommand $consoleCommand) =>
                $consoleCommand->handler->getDeclaringClass()->getName() === $command[0]
                    && $consoleCommand->handler->getName() === $command[1],
        );

        $callable = new CommandBusMiddlewareCallable(
            fn (object $command) => $this->container->get($middlewareClass)($command, $callable),
        );
    }
}
