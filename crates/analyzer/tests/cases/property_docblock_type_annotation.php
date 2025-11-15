<?php

class ConfigurationManager
{
    public function __construct(
        /** @var non-empty-string Directory path that must not be empty */
        private readonly string $configDirectory,
        /** @var positive-int Port number must be positive */
        private readonly int $port,
    ) {}

    /**
     * @param non-empty-string $path
     */
    private function validatePath(string $path): bool
    {
        return true;
    }

    /**
     * @param positive-int $port
     */
    private function validatePort(int $port): bool
    {
        return $port > 0;
    }

    public function validate(): bool
    {
        $validPath = $this->validatePath($this->configDirectory);
        $validPort = $this->validatePort($this->port);

        return $validPath && $validPort;
    }
}

class TemplateRepository
{
    public function __construct(
        /** @var non-empty-string $templateDir */
        private readonly string $templateDir,
    ) {}

    /**
     * @param non-empty-string $directory
     * @return list<string>
     */
    private function readDirectory(string $directory): array
    {
        return [];
    }

    /**
     * @return list<string>
     */
    public function findAll(): array
    {
        return $this->readDirectory($this->templateDir);
    }
}
